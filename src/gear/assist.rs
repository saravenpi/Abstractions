//! The assists: a gentle nose-up bias when the ground comes up steep and fast, and the wings
//! that open toward the view.
//!
//! The ground assist is added to what the pilot asks for and is bounded, so it softens unfair
//! impact angles and never takes the controls away: looking or pushing further down still goes
//! further down.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::aim::wrap;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::{FLARE_PITCH, FLARE_STEEP, FLARE_TIME, OPEN_DONE, OPEN_HOLD, OPEN_TURN};
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

/// Returns how far the path over the ground of `state` lies from its nose heading, in radians
/// toward the nose, or nothing for a body with no way over the ground.
fn gap(state: &FlightState) -> Option<f32> {
    let flat = state.velocity.with_y(0.0);
    (flat.length() > 1e-3).then(|| wrap(state.servo.yaw - (-flat.x).atan2(-flat.z)))
}

/// Returns true while the wings of `state` open toward the heading they were asked open on:
/// for up to [`OPEN_HOLD`] seconds after they were, until the path is on that heading, and only
/// for a path within a quarter turn of it. The nose holds that heading meanwhile.
pub(super) fn opens(state: &FlightState) -> bool {
    let turning = gap(state).is_some_and(|gap| (OPEN_DONE..=FRAC_PI_2).contains(&gap.abs()));
    state.mode.winged() && state.servo.clock < OPEN_HOLD && turning
}

/// Returns the velocity of `state` after `h` seconds under `gravity` of wings that open, see
/// [`opens`]: the path over the ground turns toward the nose heading, its speed kept, at up to
/// [`OPEN_TURN`], so the wings catch the air the way they open, toward where the view aimed the
/// folded body. A path more than a quarter turn off the nose, which the folded body only heads
/// against when it runs away from the view, is left alone, as is every path once open.
pub(super) fn opening(state: &FlightState, gravity: f32, h: f32) -> Vec3 {
    let (v, flat) = (state.velocity, state.velocity.with_y(0.0));
    let Some(gap) = gap(state).filter(|_| opens(state)) else {
        return v;
    };
    let rate = OPEN_TURN.0.min(OPEN_TURN.1 * gravity / flat.length());
    Quat::from_rotation_y(gap.clamp(-rate * h, rate * h)) * flat + Vec3::Y * v.y
}
