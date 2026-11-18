//! The boot boosters: a fall brake with the wings folded, thrust with the wings spread.

use bevy::prelude::*;

use super::aero::{bite, drag, grip};
use super::aim::look_yaw;
use super::course::ahead;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::{
    BOOT_AIR_ACCEL, BOOT_AIR_DAMP, BOOT_DESCENT, BOOT_DESCENT_RANGE, BRAKE_MAX_G, FALL_AIR_ACCEL,
    REF_GRAVITY, SUPPORT_FADE, SUPPORT_G, SUPPORT_RUN, SUPPORT_RUN_SPEED, THRUST_G, VECTOR_LIMIT,
    VECTORING,
};
use crate::render::smoothstep as smooth;

/// Returns the descent speed in metres per second the boots brake a fall to under `gravity`.
pub fn descent_target(gravity: f32) -> f32 {
    let (low, high) = BOOT_DESCENT_RANGE;
    (BOOT_DESCENT * (gravity / REF_GRAVITY).max(0.0).sqrt()).clamp(low, high)
}

/// Returns the upward acceleration of the braking boots at vertical speed `climb`.
///
/// One clamped straight line: nothing while rising, a share of the weight while sinking slower
/// than the target, more than the weight while sinking faster. The fall eases to the target from
/// both sides and cannot cross it, and a jump is never extended.
pub(super) fn brake(climb: f32, gravity: f32) -> f32 {
    let target = descent_target(gravity);
    (gravity * (1.0 - (target + climb) / target)).clamp(0.0, BRAKE_MAX_G * gravity)
}

/// Returns the horizontal direction the sticks ask for, around the look heading.
fn wish(state: &FlightState, input: &GearInput) -> Vec3 {
    let stick = Vec3::new(input.roll, 0.0, input.pitch).clamp_length_max(1.0);
    Quat::from_rotation_y(look_yaw(input.look, state.servo.yaw)) * stick
}

/// Returns the acceleration of the boots with the wings folded: the brake and a modest drift.
fn braking(state: &FlightState, input: &GearInput) -> Vec3 {
    let flat = state.velocity.with_y(0.0);
    let drift = wish(state, input) * BOOT_AIR_ACCEL - flat * BOOT_AIR_DAMP;
    drift + Vec3::Y * brake(state.velocity.y, input.gravity)
}

/// Returns the unit direction of the thrust: the nose, turned a little toward the course.
fn thrust_direction(state: &FlightState) -> Vec3 {
    let nose = state.nose();
    let lead = (ahead(state) - nose) * VECTORING;
    (nose + lead.clamp_length_max(VECTOR_LIMIT)).normalize_or(nose)
}

/// Returns the acceleration of the boots with the wings spread, at a height where they have
/// the grip `sigma`.
///
/// Thrust along the nose, plus a vertical top-up at low airspeed that carries most of the weight
/// until the wings do. The top-up never brings the vertical force to the weight. Thrust and
/// top-up thin with height exactly as the drag does.
fn flying(state: &FlightState, input: &GearInput, sigma: f32) -> Vec3 {
    let thrust = thrust_direction(state) * (THRUST_G * input.gravity * sigma);
    let carried = thrust.y + drag(state, state.velocity, bite(input)).y;
    let flat = state.velocity.with_y(0.0).length();
    let run = smooth(SUPPORT_RUN_SPEED.0, SUPPORT_RUN_SPEED.1, flat);
    let cap = (SUPPORT_G + SUPPORT_RUN * run) * input.gravity;
    let fade = 1.0 - smooth(SUPPORT_FADE.0, SUPPORT_FADE.1, state.velocity.length());
    thrust + Vec3::Y * ((cap - carried).clamp(0.0, cap) * fade * sigma)
}

/// Returns the acceleration the boots and the leaning body give in the air of `input`.
///
/// The brake and the thrust blend with the wing spread, and both scale with the booster spool,
/// so that switching anything mid-air changes the push smoothly. The boots follow the height
/// alone, so they brake as well on a planet without air. Thrust along the nose needs wings that
/// carry: where the air is thinner than at home the spread wings hand that much of the push
/// back to the brake, so the top speed is the same in thin air and no flight runs away without
/// drag. The lean of a falling body is a force of the air and fades with it.
pub(super) fn push(state: &FlightState, input: &GearInput) -> Vec3 {
    let (spread, power) = (state.spread(), state.power());
    let (sigma, air) = (grip(input.density), bite(input));
    let brake = braking(state, input) * sigma;
    let lean = FALL_AIR_ACCEL * air.min(1.0) * (1.0 - spread) * (1.0 - power);
    let winged = spread * input.air.clamp(0.0, 1.0);
    brake.lerp(flying(state, input, sigma), winged) * power + wish(state, input) * lean
}
