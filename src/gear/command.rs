//! What the pilot asks of the wings: a flight path angle and a bank, never a rate.
//!
//! This is stabilised flight, as in the angle mode of a flight controller. A held key asks for
//! an attitude inside fixed limits, and let go the wings come level on the best glide. Nothing
//! asks for more than the limits, so the body never rolls over and never loops. Near the stall
//! the limits close in: the nose is held down and the bank kept shallow until the speed is back.

use bevy::prelude::*;

use super::aero::bite;
use super::assist::flare;
use super::input::GearInput;
use super::path::{climb, path_offset};
use super::servo::ease;
use super::state::FlightState;
use super::tune::{
    BANK_GAIN, BANK_LIMIT, BANK_SPEED, CLIMB_LIMIT, COMMAND_RATE, DIVE_LIMIT, MAX_PITCH,
    STALL_BANK, STALL_PITCH, STALL_SPEED, WING_DRAG, YAW_TURN,
};
use crate::render::smoothstep as smooth;

/// Returns the flight path angle of the best glide of the spread wing, in radians: about 13
/// degrees down, a glide of `(dorsal - nose) / (2 * sqrt(dorsal * nose))` to one.
fn best_glide() -> f32 {
    let ratio = (WING_DRAG.y - WING_DRAG.z) / (2.0 * (WING_DRAG.y * WING_DRAG.z).sqrt());
    -ratio.recip().atan()
}

/// Returns how clear of the stall the wings fly, from 0 at the stall speed to 1 clear of it.
/// Under power the boots carry the body, and the protection lets go with their spool.
fn safe(state: &FlightState) -> f32 {
    let speed = state.velocity.length();
    smooth(STALL_SPEED.0, STALL_SPEED.1, speed).max(state.power())
}

/// Returns the flight path angle the keys and the view ask for.
///
/// It is the view while it steers, else the trim: the best glide, or level flight under power.
/// The back key lifts it toward [`CLIMB_LIMIT`] and the forward key lowers it toward
/// [`DIVE_LIMIT`], the ground assist flares it, and near the stall it is never higher than
/// [`STALL_PITCH`].
fn wanted(state: &FlightState, input: &GearInput) -> f32 {
    let trim = best_glide().lerp(0.0, state.power());
    let base = input.aim.map_or(trim, |aim| aim.y);
    let base = base.clamp(DIVE_LIMIT, CLIMB_LIMIT);
    let stick = input.pitch.clamp(-1.0, 1.0);
    let keyed = match stick >= 0.0 {
        true => base.lerp(CLIMB_LIMIT, stick),
        false => base.lerp(DIVE_LIMIT, -stick),
    };
    let ceiling = STALL_PITCH.lerp(CLIMB_LIMIT, safe(state));
    (keyed.min(ceiling) + flare(state, input)).min(CLIMB_LIMIT)
}

/// Returns the asked flight path angle after `h` seconds.
///
/// Folded it rides the flight path. As the wings spread it eases from there toward [`wanted`]
/// at [`COMMAND_RATE`], so spreading them mid-air flies on along the path the body had.
pub(super) fn command(state: &FlightState, input: &GearInput, h: f32) -> f32 {
    let path = climb(state).clamp(-MAX_PITCH, MAX_PITCH);
    if !state.mode.winged() {
        return path;
    }
    let asked = state.servo.climb;
    let eased = asked + (wanted(state, input) - asked) * ease(COMMAND_RATE, h);
    path.lerp(eased, state.spread())
}

/// Returns the bank the keys and the view ask for, positive with the right wing down.
///
/// A full roll key holds [`BANK_LIMIT`]; a view to one side banks toward it. Near the stall the
/// limit shrinks to [`STALL_BANK`] of itself.
pub(super) fn bank(state: &FlightState, input: &GearInput) -> f32 {
    let limit = BANK_LIMIT * STALL_BANK.lerp(1.0, safe(state));
    let view = input
        .aim
        .map_or(0.0, |aim| BANK_GAIN * path_offset(state, aim.x));
    (view + input.roll.clamp(-1.0, 1.0) * limit).clamp(-limit, limit)
}

/// Returns the heading the wings fly along: the view while it steers, else the flight path, or
/// the nose heading for a path that has none.
pub(super) fn course(state: &FlightState, input: &GearInput) -> f32 {
    let yaw = state.servo.yaw;
    input
        .aim
        .map_or_else(|| yaw + path_offset(state, yaw), |aim| aim.x)
}

/// Returns the velocity of `state` turned flat around the local up by the yaw stick for `h`
/// seconds, at up to [`YAW_TURN`]. The flight path turns, and the nose follows it. Only spread
/// wings at flying speed in air turn it, and the speed is kept.
pub(super) fn yawed(state: &FlightState, input: &GearInput, h: f32) -> Vec3 {
    let speed = state.velocity.length();
    let grip = state.spread() * smooth(BANK_SPEED.0, BANK_SPEED.1, speed) * bite(input).min(1.0);
    let rate = -input.yaw.clamp(-1.0, 1.0) * YAW_TURN * grip;
    Quat::from_rotation_y(rate * h) * state.velocity
}
