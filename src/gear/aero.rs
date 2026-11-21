//! Air on the body: one drag per body axis. The drag across the spread wing is the lift.
//!
//! Every term opposes the velocity component it acts on, so air can only take energy away.

use bevy::prelude::*;

use super::input::GearInput;
use super::state::FlightState;
use super::tune::{AERO, BELLY_DRAG, BODY_DRAG, SEA_DENSITY, THIN_AIR, WING_DRAG};

/// Returns how well the gear bites air of `density` kilograms per cubic metre: 0 in vacuum,
/// 1 at sea level.
///
/// Lift, drag, thrust and brake all scale with it. It is not the plain density ratio: the
/// scale height of these small planets is a few hundred metres, and a suit that lost half its
/// lift over the first hill would not be a suit. The grip holds up through the lower air and
/// then lets go: `1 - exp(-ratio / THIN_AIR)`. Because thrust and drag share it, the top speed is
/// the same at any height, and the ceiling is where the wing can no longer carry the weight.
pub(super) fn grip(density: f32) -> f32 {
    1.0 - (-(density / SEA_DENSITY).max(0.0) / THIN_AIR).exp()
}

/// Returns how well the wings and the body bite the air `input` describes: the [`grip`] at this
/// height times the thickness of the air of the planet. Zero without air, where the wings give
/// nothing; above 1 in thick air, which carries more and slows more.
///
/// Every force of the air is a product with it and nothing is divided by it.
pub(super) fn bite(input: &GearInput) -> f32 {
    grip(input.density) * input.air.max(0.0)
}

/// Returns the drag coefficients along the right, dorsal and nose axes of `state`: the folded
/// body, streamlined upright and spread flat across the air as far as it lies prone, opening
/// into the wing as far as it is spread and lies along the flight frame.
///
/// The wing is the fabric between the arms and the legs, and it flies only as far as the body
/// lies along the frame it is read in, which is the attitude the body is drawn at
/// (`FlightState::pose`). An upright body with its arms out meets a fall edge on and does not
/// glide; laid prone it does. So the air carries exactly the body the eye sees, through the
/// whole opening.
fn coefficients(state: &FlightState) -> Vec3 {
    let prone = state.prone.clamp(0.0, 1.0);
    let body = BODY_DRAG.lerp(BELLY_DRAG, prone);
    Vec3::splat(body).lerp(WING_DRAG, state.spread() * prone)
}

/// Returns the acceleration air of grip `sigma` gives a body flying at `velocity`.
pub(super) fn drag(state: &FlightState, velocity: Vec3, sigma: f32) -> Vec3 {
    let frame = state.frame();
    let local = frame.inverse() * velocity;
    frame * (-(coefficients(state) * local) * (AERO * sigma * velocity.length()))
}

/// Returns `velocity` after `h` seconds of drag in air of grip `sigma`.
///
/// The update is implicit on each body axis: it cannot overshoot and never adds speed.
pub(super) fn slow(state: &FlightState, velocity: Vec3, sigma: f32, h: f32) -> Vec3 {
    let frame = state.frame();
    let rate = AERO * sigma * velocity.length() * h;
    frame * ((frame.inverse() * velocity) / (Vec3::ONE + coefficients(state) * rate))
}

/// Returns the lift acceleration per radian of angle of attack the spread wing gives at `speed`.
pub(super) fn lift_slope(speed: f32, sigma: f32) -> f32 {
    AERO * sigma * speed * speed * (WING_DRAG.y - WING_DRAG.z)
}

/// Returns the angle of attack of the best glide of the spread wing, in radians: where the
/// lift over the drag of [`WING_DRAG`] is largest.
pub(super) fn best_aoa() -> f32 {
    (WING_DRAG.z / WING_DRAG.y).sqrt().atan()
}

/// Returns the angle of attack of `velocity` on the flight frame, zero at rest.
pub(super) fn angle_of_attack(state: &FlightState, velocity: Vec3) -> f32 {
    if velocity.length_squared() < 1.0 {
        return 0.0;
    }
    let local = state.frame().inverse() * velocity;
    (-local.y).atan2(-local.z)
}
