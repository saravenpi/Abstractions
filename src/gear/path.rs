//! The flight path of a body: how steep it runs and how far its heading lies from another.

use super::aim::wrap;
use super::state::FlightState;
use crate::render::smoothstep as smooth;

/// Shares of the speed that is ground speed between which the flight path has a heading.
const STEEP: (f32, f32) = (0.2, 0.5);

/// Cosines of the angle between the nose and the velocity between which the body flies nose
/// first: from across the air, where it only falls, to 60 degrees off, a deep stall.
const NOSE_FIRST: (f32, f32) = (0.0, 0.5);

/// Returns how far the flight path heading lies from `yaw`, zero for a body with no ground
/// speed. A path that goes nearly straight up or down has no heading to be off, so the offset
/// fades there. It fades too for a body that does not fly nose first, such as one sinking
/// through a stall: its path may read as a half turn off, and chasing that would swing the
/// heading from one side to the other in a frame.
pub(super) fn path_offset(state: &FlightState, yaw: f32) -> f32 {
    let along = state.velocity;
    let flat = along.with_y(0.0).length();
    if flat < 1.0 {
        return 0.0;
    }
    let steep = smooth(STEEP.0, STEEP.1, flat / along.length());
    let ahead = state.nose().dot(state.velocity) / state.velocity.length();
    let first = smooth(NOSE_FIRST.0, NOSE_FIRST.1, ahead);
    wrap((-along.x).atan2(-along.z) - yaw) * steep * first
}

/// Returns the angle of the flight path of `state` above the horizon, from straight down to
/// straight up, or the pitch itself at rest.
pub(super) fn climb(state: &FlightState) -> f32 {
    let v = state.velocity;
    if v.length_squared() < 1e-6 {
        return state.pitch;
    }
    v.y.atan2(v.with_y(0.0).length())
}
