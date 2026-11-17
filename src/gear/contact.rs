//! What a collision does to the flight: the controller slides the body, the gear takes the result.

use bevy::prelude::*;

use super::state::FlightState;
use super::tune::SCRUB;

/// Returns the velocity a body flying at `before` keeps after the physics slid it to `moved`.
///
/// Sliding already removes the part of the velocity that went into the surface. A hit also
/// scrubs a share of what is left, in proportion to the sine of the impact angle, so a glancing
/// touch costs almost nothing and a square hit costs a lot.
pub fn deflect(before: Vec3, moved: Vec3) -> Vec3 {
    let speed = before.length();
    if speed < 1e-3 {
        return moved;
    }
    let sine = ((before - moved).length() / speed).min(1.0);
    moved * (1.0 - SCRUB * sine)
}

/// Returns `state` after the controller moved the body and the physics left it `moved` metres
/// per second, in the upright frame.
///
/// Call it after move and slide in every mode that drives. The speed lost into the surface is
/// remembered until the next step, where a touchdown reads it to tell a slide from a landing roll.
pub fn slid(state: &FlightState, moved: Vec3) -> FlightState {
    let mut out = *state;
    let lost = (state.velocity - moved).length();
    out.velocity = match state.mode.airborne() {
        true => deflect(state.velocity, moved),
        false => moved,
    };
    out.servo.impact = if lost > 0.5 { lost } else { 0.0 };
    out
}
