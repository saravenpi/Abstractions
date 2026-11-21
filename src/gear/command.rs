//! What the pilot asks of the wings: rates from the keys, an attitude from the view.
//!
//! The keys fly the wings as a pilot flies them, in the frame of the body. The roll keys roll
//! it on round its nose (see `roll`), the pitch keys pull toward its back or push toward its
//! belly as hard as the wing may (see `aim`), and the yaw keys turn the path round its back,
//! which is a flat turn with the wings level. Nothing
//! limits the attitude: held, they roll the body over and loop it. Let go, the wings come
//! level and the flight path eases back to the best glide, and near the stall the nose goes
//! down. While the steer button is held the view asks for an attitude inside fixed limits,
//! however the body and the view are read (see `path::sighted`).

use bevy::prelude::*;

use super::aero::bite;
use super::aim::wrap;
use super::assist::flare;
use super::input::GearInput;
use super::path::{bounded, climb, mirrored, path_offset, sighted};
use super::servo::ease;
use super::state::FlightState;
use super::tune::{
    BANK_GAIN, BANK_LIMIT, BANK_SPEED, CLIMB_LIMIT, COMMAND_RATE, DIVE_LIMIT, LEVEL, LEVEL_PITCH,
    LIMIT_SPEED, STALL_BANK, STALL_PITCH, STALL_SPEED, WING_DRAG, YAW_TURN,
};
use crate::render::smoothstep as smooth;

/// Rate per second at which the asked flight path angle rides the path while a pitch key is
/// held at flying speed, so the nose stays on the path the wing turns. Slower it lets go, as
/// the air lets go of the body: under [`LIMIT_SPEED`] a stalled body does not whip round after
/// a path that points nowhere.
const RIDE_RATE: f32 = 20.0;

/// Returns the flight path angle of the best glide of the spread wing, in radians: about 13
/// degrees down, a glide of `(dorsal - nose) / (2 * sqrt(dorsal * nose))` to one.
fn best_glide() -> f32 {
    let ratio = (WING_DRAG.y - WING_DRAG.z) / (2.0 * (WING_DRAG.y * WING_DRAG.z).sqrt());
    -ratio.recip().atan()
}

/// Returns how clear of the stall the wings fly, from 0 at the stall speed to 1 clear of it.
/// Under power the boots carry the body, and the protection lets go with their spool.
pub(super) fn safe(state: &FlightState) -> f32 {
    let speed = state.velocity.length();
    smooth(STALL_SPEED.0, STALL_SPEED.1, speed).max(state.power())
}

/// Returns the flight path angle the wings ease toward while no pitch key is held.
///
/// It is the view while it steers, inside [`DIVE_LIMIT`] and [`CLIMB_LIMIT`], else the trim:
/// the best glide, or level flight under power. The ground assist flares it, and near the stall
/// it is never higher than [`STALL_PITCH`]. A body whose pitch reads past the vertical reads
/// it there too.
fn wanted(state: &FlightState, input: &GearInput) -> f32 {
    let trim = best_glide().lerp(0.0, state.power());
    let base = sighted(state, input).map_or(trim, |aim| aim.y);
    let base = base.clamp(DIVE_LIMIT, CLIMB_LIMIT);
    let ceiling = STALL_PITCH.lerp(CLIMB_LIMIT, safe(state));
    mirrored(
        state,
        (base.min(ceiling) + flare(state, input)).min(CLIMB_LIMIT),
    )
}

/// Returns the share of the self-level the flight path angle gets: [`LEVEL_PITCH`] clear of
/// the stall, where let go the wings fly on along the path the keys left, and all of it at the
/// stall, where the nose comes down.
fn held(state: &FlightState) -> f32 {
    LEVEL_PITCH.lerp(1.0, 1.0 - safe(state))
}

/// Returns the asked flight path angle after `h` seconds.
///
/// Folded it rides the flight path. On the wings a held pitch key flies by rate, so the asked
/// angle rides the path too, at [`RIDE_RATE`], and let go it eases from there toward
/// [`wanted`]: at [`COMMAND_RATE`] for the view, at [`LEVEL`] of it for the self-level, see
/// [`held`]. As the
/// wings spread it eases from the path, so spreading them mid-air flies on along the path the
/// body had.
pub(super) fn command(state: &FlightState, input: &GearInput, h: f32) -> f32 {
    let path = climb(state);
    if !state.mode.winged() {
        return bounded(state, path);
    }
    let speed = state.velocity.length();
    let (goal, rate) = match input.pitch.abs() > f32::EPSILON {
        true => (
            path,
            RIDE_RATE * smooth(LIMIT_SPEED.0, LIMIT_SPEED.1, speed),
        ),
        false => (
            wanted(state, input),
            input
                .aim
                .map_or(COMMAND_RATE * LEVEL * held(state), |_| COMMAND_RATE),
        ),
    };
    let asked = state.servo.climb;
    path.lerp(asked + wrap(goal - asked) * ease(rate, h), state.spread())
}

/// Returns the bank a view that steers asks for, counted from the level bank and positive with
/// the right wing down: toward the side it looks to, at most [`BANK_LIMIT`], and near the stall
/// [`STALL_BANK`] of that. None while the view steers nothing.
pub(super) fn bank(state: &FlightState, input: &GearInput) -> f32 {
    let limit = BANK_LIMIT * STALL_BANK.lerp(1.0, safe(state));
    let view = sighted(state, input).map_or(0.0, |aim| BANK_GAIN * path_offset(state, aim.x));
    view.clamp(-limit, limit)
}

/// Returns the heading the wings fly along: the view while it steers, else the flight path, or
/// the nose heading for a path that has none.
pub(super) fn course(state: &FlightState, input: &GearInput) -> f32 {
    let yaw = state.servo.yaw;
    sighted(state, input).map_or_else(|| yaw + path_offset(state, yaw), |aim| aim.x)
}

/// Returns the velocity of `state` turned by the yaw stick for `h` seconds, at up to
/// [`YAW_TURN`], toward the right wing for a stick to the right.
///
/// The turn is round the dorsal axis of the body, as every key works in the frame of the body:
/// flat with the wings level, and toward the same wing on its back, on edge and with the nose
/// straight up or down. The flight path turns, and the nose follows it. Only spread wings at
/// flying speed in air turn it, and the speed is kept.
pub(super) fn yawed(state: &FlightState, input: &GearInput, h: f32) -> Vec3 {
    let speed = state.velocity.length();
    let grip = state.spread() * smooth(BANK_SPEED.0, BANK_SPEED.1, speed) * bite(input).min(1.0);
    let rate = -input.yaw.clamp(-1.0, 1.0) * YAW_TURN * grip;
    Quat::from_axis_angle(state.frame() * Vec3::Y, rate * h) * state.velocity
}
