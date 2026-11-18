//! The flight path of a body: how steep it runs and how far its heading lies from another.

use bevy::prelude::*;

use super::aim::wrap;
use super::state::FlightState;
use crate::render::smoothstep as smooth;

/// Shares of the speed that is ground speed between which the flight path has a heading.
const STEEP: (f32, f32) = (0.2, 0.5);

/// Cosines of the angle between the nose and the velocity between which the body flies nose
/// first: from across the air, where it only falls, to 60 degrees off, a deep stall.
const NOSE_FIRST: (f32, f32) = (0.0, 0.5);

/// Returns how far the flight path heading lies from `yaw`, zero for a body with no ground
/// speed. A body on its back, past the vertical of a loop, flies against its heading, so its
/// path is read the other way round, and a path that goes straight up or down has no heading
/// to be off: the offset fades there, or the top of a loop would read as a half turn to make.
/// It fades too for a body that does not fly nose first: one that fell through a stall on its
/// back moves tail first, its path reads as a half turn off, and chasing that swings the
/// heading from one side to the other in a frame.
pub fn path_offset(state: &FlightState, yaw: f32) -> f32 {
    let along = state.velocity * state.pitch.cos().signum();
    let flat = along.with_y(0.0).length();
    if flat < 1.0 {
        return 0.0;
    }
    let steep = smooth(STEEP.0, STEEP.1, flat / along.length());
    let ahead = state.nose().dot(state.velocity) / state.velocity.length();
    let first = smooth(NOSE_FIRST.0, NOSE_FIRST.1, ahead);
    wrap((-along.x).atan2(-along.z) - yaw) * steep * first
}

/// Returns the angle of the flight path of `state` above the horizon, on the turn nearest its
/// pitch, or the pitch itself at rest.
///
/// On the wings the ground speed counts along the nose heading, negative when the body flies
/// backward over the top of a loop, so the angle runs on smoothly through the vertical on
/// either side. Folded, the heading follows the view and not the path: the ground speed is
/// read forward while the frame is upright and backward while it is on its back.
pub(super) fn climb(state: &FlightState) -> f32 {
    let (v, yaw) = (state.velocity, state.servo.yaw);
    if v.length_squared() < 1e-6 {
        return state.pitch;
    }
    let flat = match state.mode.winged() {
        true => v.dot(Vec3::new(-yaw.sin(), 0.0, -yaw.cos())),
        false => v.with_y(0.0).length() * state.pitch.cos().signum(),
    };
    state.pitch + wrap(v.y.atan2(flat) - state.pitch)
}
