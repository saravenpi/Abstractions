//! Where the attitude wants to go: the frame along the flight path, the pull of the wing on it.
//!
//! The pitch keys work through the pull alone: the wing turns the path, and the frame follows
//! the path. So a loop is flown, not turned, and a wing too slow to turn the path falls out.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::aero::{best_aoa, bite, lift_slope};
use super::assist::opens;
use super::command::{course, safe};
use super::input::GearInput;
use super::mode::Mode;
use super::path::{bounded, climb, path_offset, sighted};
use super::roll::{facing, roll, sense};
use super::state::FlightState;
use super::tune::{
    AOA_MAX, AOA_MIN, FALL_ARCH, FOLLOW_SPEED, LIMIT_SPEED, MAX_LOAD, PATH_TIME, PRONE_SPEED,
    PULL_OUT, PULL_TURN, TRIM_LOAD, TRIM_MAX,
};
use crate::render::smoothstep as smooth;

/// Largest share in radians of the gap between the flight frame and its target that the pull
/// makes up. The frame lags a path that turns, and at speed the wing pulls its whole load with
/// a few hundredths of a radian, about that lag: without this a fast pull-out pulls one g
/// where it may pull four.
const LAG: f32 = 0.3;

/// Cosines of the pitch between which a body on its wings is laid flat whatever its speed:
/// with the nose near the vertical, standing it up has no shortest way round. The same cosines
/// of the flight path free a banked pull from [`super::tune::TURN_CAP`]: such a path has no
/// heading to turn.
pub(super) const STEEP: (f32, f32) = (0.2, 0.5);

/// How far the back faces the sky between which the trim carries the weight: none on the back.
const CARRY: (f32, f32) = (0.0, 0.3);

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

/// Returns the heading and the flight path angle the body is asked to fly along.
///
/// On the wings it is the command of the pilot, see `command`. In a fall the gear took over it
/// is the flight path, so the folded body lies along its momentum. Otherwise it is the aim the
/// caller gave, or the angles of the look direction.
pub(super) fn asked(state: &FlightState, input: &GearInput) -> Vec2 {
    if state.mode.winged() || state.mode == Mode::Fall {
        return Vec2::new(course(state, input), state.servo.climb);
    }
    let level = look_yaw(input.look, state.servo.yaw);
    sighted(state, input)
        .unwrap_or_else(|| Vec2::new(level, input.look.normalize_or_zero().y.asin()))
}

/// Returns the largest angle of attack the wing may take at this speed: the stall angle, and
/// less at high speed, where the same angle would pull more than [`MAX_LOAD`] g.
///
/// Gliding with the pitch and roll keys let go it is no more than the angle of the best glide: a
/// hands-off wing too slow for its path drops its nose and gathers speed, where one held at
/// the stall angle would sink on at that angle. Under power the thrust gathers the speed. A
/// wing clear of the stall with its path more than [`PULL_OUT`] under the asked one lets go of
/// that limit and pulls out of the dive at the load it may take. A held key takes the wing to
/// the stall angle: the pilot may stall it.
fn lead(state: &FlightState, input: &GearInput) -> f32 {
    let slope = lift_slope(state.velocity.length(), bite(input));
    let stall = AOA_MAX.min(MAX_LOAD * input.gravity / slope.max(1e-3));
    let held = input.pitch.abs().max(input.roll.abs()).min(1.0);
    let under = (state.servo.climb - climb(state)) * sense(state);
    let dive = smooth(0.0, PULL_OUT, under) * safe(state, input);
    let calm = (1.0 - held) * state.spread() * (1.0 - state.power()) * (1.0 - dive);
    stall.lerp(stall.min(best_aoa()), calm)
}

/// Returns the pull target, toward the back of the body.
///
/// Let go, it is the nose-up trim that carries the weight at this speed and bank, none on the
/// back, and the pull that turns the flight path onto the asked angle within [`PATH_TIME`]. The
/// back key pulls what turns the path toward the back at [`PULL_TURN`], as far as [`lead`], the
/// most the wing may take: too slow, the wing stalls before the path comes round. The forward
/// key unloads the wing
/// to no lift at all, so the weight alone curves the path down into a dive, as steep as the
/// vertical and never past it. Upside down a pull still turns the path toward the back.
fn pull(state: &FlightState, input: &GearInput) -> f32 {
    let speed = state.velocity.length();
    let level = state.velocity.with_y(0.0).length() / speed.max(1e-3);
    let carry = smooth(CARRY.0, CARRY.1, facing(state));
    let load = (level / state.bank.cos().abs().max(0.1)).min(TRIM_LOAD) * carry;
    let turn = wrap(asked(state, input).y - climb(state)).clamp(-1.0, 1.0) * speed / PATH_TIME;
    let slope = lift_slope(speed, bite(input)).max(1e-3);
    let lift = turn * sense(state) / slope;
    let trim = (input.gravity * load / slope).min(TRIM_MAX) + lift;
    let (stick, most) = (input.pitch.clamp(-1.0, 1.0), lead(state, input));
    let back = (state.frame() * Vec3::Y).y;
    let turning = (speed * PULL_TURN + input.gravity * back) / slope;
    let keyed = match stick >= 0.0 {
        true => trim.lerp(turning.clamp(0.0, most), stick),
        false => trim.lerp(0.0, -stick),
    };
    (keyed * state.spread()).clamp(-AOA_MIN, most)
}

/// Returns the pitch target: the asked angle, held near the flight path at speed.
///
/// With the pull, the nose may lead the path toward the back by [`lead`] at most; rolled over
/// that side lies the other way. The limits do not wait for the wings: with them folded the
/// flight frame already lies along the fall, so that spreading them never meets the air flat
/// on. Folded, the frame stays within [`super::tune::MAX_PITCH`] of the horizon.
fn pitch(state: &FlightState, input: &GearInput, pull: f32) -> f32 {
    let wanted = asked(state, input).y;
    let (speed, path) = (state.velocity.length(), climb(state));
    let room = (lead(state, input) - pull).max(0.0);
    let over = smooth(-1.0, 1.0, -sense(state));
    let held = wanted.clamp(
        path - AOA_MIN.lerp(room, over),
        path + room.lerp(AOA_MIN, over),
    );
    let weight = smooth(LIMIT_SPEED.0, LIMIT_SPEED.1, speed);
    let target = wanted + (held - wanted) * weight;
    match state.mode.winged() {
        true => target,
        false => bounded(state, target),
    }
}

/// Returns the attitude target for this state and input.
///
/// With the wings folded it is upright along the asked heading. As they spread and the airspeed
/// builds, the nose heading follows the flight path, the roll keys roll the wings and the view
/// banks them, the banked lift turns the path, and the body lies prone as far as they spread,
/// see [`lain`]. In a fall the gear took over the body lies flat across the fall as fast as on
/// the wings, folded from a glide it stays prone, belly to the air, and it heads where the
/// view aims it, not along the path it had: the wings open again toward the view. While they
/// open the nose holds the heading they open on and the path turns onto it, see
/// [`opens`]; then the nose follows the path again.
pub(super) fn aim(state: &FlightState, input: &GearInput) -> Aim {
    let (spread, speed) = (state.spread(), state.velocity.length());
    let opening = opens(state);
    let heading = match opening {
        true => state.servo.yaw,
        false => asked(state, input).x,
    };
    let offset = path_offset(state, heading);
    let flat = state.velocity.with_y(0.0).length();
    let along = spread.max(f32::from(u8::from(state.mode == Mode::Fall)));
    let follow =
        spread * smooth(FOLLOW_SPEED.0, FOLLOW_SPEED.1, flat) * f32::from(u8::from(!opening));
    let pull = pull(state, input);
    let pitch = pitch(state, input, pull);
    let lag = (pitch - state.pitch).clamp(-LAG, LAG) * sense(state);
    Aim {
        pull: pull + lag * spread,
        yaw: heading + follow * offset,
        pitch,
        bank: roll(state, input),
        prone: lain(state, input, (along, speed)),
    }
}

/// Returns how prone the body is asked to lie, for a body `along` its path by that share at
/// `speed` metres per second.
///
/// Spread, the wings lay it prone along the flight frame as far as they are spread, whatever
/// its speed: a glide starts chest down with the arms and legs out. A body the gear took over
/// lies prone as its speed builds, at any speed on its back or with the nose near the vertical.
/// A body that falls with no gear in use lies in the belly-down arch of a fast fall, see
/// [`FALL_ARCH`], so the wings open from the arch it is drawn in.
fn lain(state: &FlightState, input: &GearInput, (along, speed): (f32, f32)) -> f32 {
    let steep = 1.0 - smooth(STEEP.0, STEEP.1, state.pitch.cos());
    let flat = smooth(PRONE_SPEED.0, PRONE_SPEED.1, speed).max(steep);
    let carried = along * flat.max(1.0 - smooth(CARRY.0, CARRY.1, facing(state)));
    let loose = state.mode == Mode::Off && !input.grounded && !input.in_water;
    let arch = smooth(FALL_ARCH.0, FALL_ARCH.1, -state.velocity.y) * f32::from(u8::from(loose));
    carried.max(state.spread()).max(arch)
}
