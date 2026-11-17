//! The ground assist: a gentle nose-up bias when the ground comes up steep and fast.
//!
//! It is added to what the pilot asks for and is bounded, so it softens unfair impact angles
//! and never takes the controls away: looking or pushing further down still goes further down.

use super::input::GearInput;
use super::state::FlightState;
use super::tune::{FLARE_PITCH, FLARE_STEEP, FLARE_TIME};
use crate::render::smoothstep as smooth;

/// Returns the nose-up bias of the ground assist: none in level flight or far from the ground.
pub(super) fn flare(state: &FlightState, input: &GearInput) -> f32 {
    let sink = -state.velocity.y;
    if sink <= 0.0 || !input.clearance.is_finite() {
        return 0.0;
    }
    let near = 1.0 - smooth(FLARE_TIME.0, FLARE_TIME.1, input.clearance.max(0.0) / sink);
    let steep = smooth(FLARE_STEEP.0, FLARE_STEEP.1, sink / state.velocity.length());
    FLARE_PITCH * near * steep * input.assist.clamp(0.0, 1.0) * state.spread()
}
