//! The flight path of a body: how steep it runs and how far its heading lies from another.
//!
//! The attitude is read as a heading, a pitch and a bank. A loop takes the pitch past the
//! vertical, so a body on its back may fly against its heading: every angle here reads the
//! path on the same side of the vertical as the pitch.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::aim::wrap;
use super::input::GearInput;
use super::state::FlightState;
use super::tune::MAX_PITCH;
use crate::render::smoothstep as smooth;

/// Shares of the speed that is ground speed between which the flight path has a heading.
const STEEP: (f32, f32) = (0.2, 0.5);

/// Cosines of the angle between the nose and the velocity between which the body flies nose
/// first: from across the air, where it only falls, to 60 degrees off, a deep stall.
const NOSE_FIRST: (f32, f32) = (0.0, 0.5);

/// Returns how far the flight path heading lies from `yaw`, zero for a body with no ground
/// speed. A path that reads past the vertical, see [`climb`], runs against the heading, so it
/// is read the other way round, and a path that goes straight up or down has no heading to be
/// off: the offset fades there, or the top of a loop would read as a half turn to make. It
/// fades too for a body that does not fly nose first, such as one sinking through a stall: its
/// path may read as a half turn off, and chasing that would swing the heading from one side to
/// the other in a frame.
pub(super) fn path_offset(state: &FlightState, yaw: f32) -> f32 {
    let along = state.velocity * climb(state).cos().signum();
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
/// read forward while the pitch reads upright and backward while it reads past the vertical.
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

/// Returns the heading and the angle above the horizon of the view that steers `state`, or
/// nothing while the view of `input` steers nothing.
///
/// This is the one place where a view becomes a command. A view is a heading and a pitch, and
/// so is the body, and each may read past the vertical, where its heading runs a half turn
/// round. The heading is returned the way the body is read, so it compares with the nose
/// heading and with [`path_offset`] whatever the two readings are. The angle is the plain one
/// above the horizon, for [`mirrored`].
pub(super) fn sighted(state: &FlightState, input: &GearInput) -> Option<Vec2> {
    let aim = input.aim?;
    let over = aim.y.cos() < 0.0;
    let across = over != (state.pitch.cos() < 0.0);
    let rise = match over {
        true => PI.copysign(aim.y) - aim.y,
        false => aim.y,
    };
    let turned = PI * f32::from(u8::from(across));
    Some(Vec2::new(aim.x + turned, rise))
}

/// Returns `angle` mirrored across the vertical when the pitch of `state` reads past it, else
/// `angle` itself: an angle above the horizon read on the side of the pitch.
pub(super) fn mirrored(state: &FlightState, angle: f32) -> f32 {
    match state.pitch.cos() < 0.0 {
        true => PI.copysign(state.pitch) - angle,
        false => angle,
    }
}

/// Returns `angle` held within [`MAX_PITCH`] of the horizon, on the side of the vertical the
/// pitch of `state` reads on.
pub(super) fn bounded(state: &FlightState, angle: f32) -> f32 {
    mirrored(state, mirrored(state, angle).clamp(-MAX_PITCH, MAX_PITCH))
}
