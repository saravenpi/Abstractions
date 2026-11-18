//! Where the attitude wants to go: the course of the keys or the view leads, the path limits.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::aero::{best_aoa, bite, lift_slope};
use super::assist::flare;
use super::course::climb;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::{
    AOA_MAX, AOA_MIN, BANK_SPEED, FOLLOW_SPEED, LIMIT_SPEED, MAX_BANK, MAX_LOAD, MAX_PITCH,
    PATH_LOAD, PITCH_AUTHORITY, PRONE_SPEED, ROLL_AUTHORITY, TRIM_LOAD, TRIM_MAX,
};
use crate::render::smoothstep as smooth;

/// Shares of the speed that is ground speed between which the flight path has a heading.
const STEEP: (f32, f32) = (0.2, 0.5);

/// Most g the wing can pull at this speed, between which a held pitch stick takes the nose off
/// the flight path. Above the band the wing turns the path and the nose stays on it, which is
/// a clean loop. Under it the wing cannot even carry the weight, the path will not come round
/// whatever is asked, and the stick turns the body through the stall instead.
const CARRY: (f32, f32) = (1.0, 1.5);

/// Largest share in radians of the gap between the flight frame and its target that the pull
/// makes up. The frame lags a path that turns, and at speed the wing pulls its whole load with
/// a few hundredths of a radian, about that lag: without this a fast pull-out pulls one g
/// where it may pull four. Kept small, so a frame turning freely through a stall does not
/// throw the nose ahead of the body.
const LAG: f32 = 0.1;

/// The attitude the gear steers toward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Aim {
    /// Heading of the nose in radians.
    pub(super) yaw: f32,
    /// Nose angle above the horizon in radians.
    pub(super) pitch: f32,
    /// Bank in radians, positive with the right wing down.
    pub(super) bank: f32,
    /// Proneness from 0 to 1.
    pub(super) prone: f32,
    /// Pull of the nose toward the dorsal side in radians.
    pub(super) pull: f32,
}

/// Returns `angle` wrapped to the half turn around zero.
pub(super) fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}

/// Returns the heading of `look` around the local up, or `fallback` when it has none.
pub(super) fn look_yaw(look: Vec3, fallback: f32) -> f32 {
    if look.x * look.x + look.z * look.z < 1e-8 {
        return fallback;
    }
    (-look.x).atan2(-look.z)
}

/// Returns the heading and the pitch asked for. On the wings it is the nose heading, which
/// [`aim`] turns onto the path, and the course of the keys: the view looks around freely.
/// Otherwise it is the view: the angles the caller gave, or the ones of the look direction.
fn asked(state: &FlightState, input: &GearInput) -> Vec2 {
    if state.mode.winged() {
        return Vec2::new(state.servo.yaw, state.servo.course);
    }
    let level = look_yaw(input.look, state.servo.yaw);
    input
        .aim
        .unwrap_or_else(|| Vec2::new(level, input.look.normalize_or_zero().y.asin()))
}

/// Returns how far the flight path heading lies from `yaw`, zero for a body with no ground
/// speed. A body on its back, past the vertical of a loop, flies against its heading, so its path is read the other way
/// round, and a path that goes straight up or down has no heading to be off: the offset fades
/// there, or the top of a loop would read as a half turn to make.
fn path_offset(state: &FlightState, yaw: f32) -> f32 {
    let along = state.velocity * state.pitch.cos().signum();
    let flat = along.with_y(0.0).length();
    if flat < 1.0 {
        return 0.0;
    }
    let steep = smooth(STEEP.0, STEEP.1, flat / along.length());
    wrap((-along.x).atan2(-along.z) - yaw) * steep
}

/// Returns the largest angle of attack the wing may take at this speed: the stall angle, and
/// less at high speed, where the same angle would pull more than [`MAX_LOAD`] g.
///
/// Gliding with the pitch stick let go it is no more than the angle of the best glide: a
/// hands-off wing too slow for its course drops its nose and gathers speed, where one held at
/// the stall angle would sink on at that angle. Under power the thrust gathers the speed.
fn lead(state: &FlightState, input: &GearInput) -> f32 {
    let slope = lift_slope(state.velocity.length(), bite(input));
    let stall = AOA_MAX.min(MAX_LOAD * input.gravity / slope.max(1e-3));
    let calm = (1.0 - input.pitch.clamp(-1.0, 1.0).abs()) * state.spread() * (1.0 - state.power());
    stall.lerp(stall.min(best_aoa()), calm)
}

/// Returns the pull target: the nose-up trim that carries the weight at this speed and bank,
/// the pull that turns the flight path onto the asked pitch at [`PATH_LOAD`], and the stick,
/// inside what the wing may take.
///
fn pull(state: &FlightState, input: &GearInput) -> f32 {
    let speed = state.velocity.length();
    let level = state.velocity.with_y(0.0).length() / speed.max(1e-3);
    let load = (level / state.bank.cos().max(0.1)).min(TRIM_LOAD);
    let wanted = asked(state, input).y + flare(state, input);
    let turn = wrap(wanted - climb(state)).clamp(-1.0, 1.0) * PATH_LOAD;
    let slope = lift_slope(speed, bite(input)).max(1e-3);
    let trim = (input.gravity * load / slope).min(TRIM_MAX) + input.gravity * turn / slope;
    let stick = input.pitch.clamp(-1.0, 1.0) * PITCH_AUTHORITY;
    ((trim + stick) * state.spread()).clamp(-AOA_MIN, lead(state, input))
}

/// Returns the pitch target: the asked pitch and the assist, held near the flight path at speed.
///
/// With the pull, the nose may lead the path by [`lead`] at most. The limits do not wait for
/// the wings: with them folded the flight frame already lies along the fall, so that spreading
/// them never meets the air flat on.
///
/// Every angle is taken on the turn nearest the pitch the body has, so the nose can follow the
/// course over the top of a loop and round again. Only a body off its wings whose caller gives
/// a look direction, which cannot pass the vertical, is held under [`MAX_PITCH`].
///
/// `hold` is how much of the path limit is left, see [`aim`]: a held pitch stick lets go of the
/// path once the wing is too slow to turn it. Without that a pull held from a glide ends in a
/// stall that mushes down with the nose tied to the falling path, and the body never comes
/// over the top.
fn pitch(state: &FlightState, input: &GearInput, (pull, hold): (f32, f32)) -> f32 {
    let near = |angle: f32| state.pitch + wrap(angle - state.pitch);
    let wanted = near(asked(state, input).y + flare(state, input));
    let (speed, path) = (state.velocity.length(), climb(state));
    let room = (lead(state, input) - pull).max(0.0);
    let held = wanted.clamp(path - AOA_MIN, path + room);
    let weight = smooth(LIMIT_SPEED.0, LIMIT_SPEED.1, speed) * hold;
    let free = wanted + (held - wanted) * weight;
    match input.aim.is_some() || state.mode.winged() {
        true => free,
        false => free.clamp(-MAX_PITCH, MAX_PITCH),
    }
}

/// Returns the attitude target for this state and input.
///
/// With the wings folded it is upright along the look heading. As they spread and the airspeed
/// builds, the nose heading follows the flight path, the roll stick banks the wings, whose
/// banked lift turns the path (a coordinated turn, no rudder), and the body lies prone.
///
/// A held pitch stick on a wing too slow to turn the path, see [`CARRY`], takes the nose off
/// the path: the pitch follows the course through the stall, and the heading and the bank stop
/// chasing a path that only falls, or the body would spin on its way over the top. The body
/// also stays prone on its flight frame however slow it gets: standing it up half way with the
/// nose straight down has no shortest way round, and it would snap.
pub(super) fn aim(state: &FlightState, input: &GearInput) -> Aim {
    let (spread, speed) = (state.spread(), state.velocity.length());
    let heading = asked(state, input).x;
    let carry = lift_slope(speed, bite(input)) * AOA_MAX / input.gravity.max(1e-3);
    let stick = input.pitch.clamp(-1.0, 1.0).abs();
    let hold = 1.0 - stick * (1.0 - smooth(CARRY.0, CARRY.1, carry));
    let offset = path_offset(state, heading) * hold;
    let flat = state.velocity.with_y(0.0).length();
    let follow = spread * smooth(FOLLOW_SPEED.0, FOLLOW_SPEED.1, flat);
    let turn = input.roll.clamp(-1.0, 1.0) * ROLL_AUTHORITY;
    let pull = pull(state, input);
    let pitch = pitch(state, input, (pull, hold));
    Aim {
        pull: pull + (pitch - state.pitch).clamp(-LAG, LAG) * spread,
        yaw: heading + follow * offset,
        pitch,
        bank: turn.clamp(-MAX_BANK, MAX_BANK) * spread * smooth(BANK_SPEED.0, BANK_SPEED.1, speed),
        prone: spread * smooth(PRONE_SPEED.0, PRONE_SPEED.1, speed).max(1.0 - hold),
    }
}
