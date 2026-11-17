//! Air on the body: one drag per body axis. The drag across the spread wing is the lift.
//!
//! Every term opposes the velocity component it acts on, so air can only take energy away.

use bevy::prelude::*;

use super::state::FlightState;
use super::tune::{AERO, BODY_DRAG, BREATH, SEA_DENSITY, WING_DRAG};

/// Returns how well the gear bites air of `density` kilograms per cubic metre: 0 in vacuum,
/// 1 at sea level.
///
/// Lift, drag, thrust and brake all scale with it. It is not the plain density ratio: the
/// scale height of these small planets is a few hundred metres, and a suit that lost half its
/// lift over the first hill would not be a suit. The grip holds up through the lower air and
/// then lets go: `1 - exp(-ratio / BREATH)`. Because thrust and drag share it, the top speed is
/// the same at any height, and the ceiling is where the wing can no longer carry the weight.
pub(super) fn grip(density: f32) -> f32 {
    1.0 - (-(density / SEA_DENSITY).max(0.0) / BREATH).exp()
}

/// Returns the drag coefficients along the right, dorsal and nose axes for a wing `spread`.
fn coefficients(spread: f32) -> Vec3 {
    Vec3::splat(BODY_DRAG).lerp(WING_DRAG, spread)
}

/// Returns the acceleration air of grip `sigma` gives a body flying at `velocity`.
pub(super) fn drag(state: &FlightState, velocity: Vec3, sigma: f32) -> Vec3 {
    let frame = state.frame();
    let local = frame.inverse() * velocity;
    frame * (-(coefficients(state.spread()) * local) * (AERO * sigma * velocity.length()))
}

/// Returns `velocity` after `h` seconds of drag in air of grip `sigma`.
///
/// The update is implicit on each body axis: it cannot overshoot and never adds speed.
pub(super) fn slow(state: &FlightState, velocity: Vec3, sigma: f32, h: f32) -> Vec3 {
    let frame = state.frame();
    let rate = AERO * sigma * velocity.length() * h;
    frame * ((frame.inverse() * velocity) / (Vec3::ONE + coefficients(state.spread()) * rate))
}

/// Returns the lift acceleration per radian of angle of attack the spread wing gives at `speed`.
pub(super) fn lift_slope(speed: f32, sigma: f32) -> f32 {
    AERO * sigma * speed * speed * (WING_DRAG.y - WING_DRAG.z)
}

/// Returns the angle of attack of `velocity` on the flight frame, zero at rest.
pub(super) fn angle_of_attack(state: &FlightState, velocity: Vec3) -> f32 {
    if velocity.length_squared() < 1.0 {
        return 0.0;
    }
    let local = state.frame().inverse() * velocity;
    (-local.y).atan2(-local.z)
}
