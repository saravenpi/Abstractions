//! The course of a body on its wings: the keys steer it, the view does not.
//!
//! The pitch keys turn the flight path the pilot asks for up or down at a steady rate, as far
//! round as a loop. Let go, the course eases back to a stable one: the best glide with the
//! wings alone, the climb or dive it was left at under power, short of the steep ones. The
//! roll keys bank the wings and the banked lift turns the path: see `aim`.

use bevy::prelude::*;

use super::aim::wrap;
use super::mode::Mode;
use super::servo::ease;
use super::state::FlightState;
use super::tune::{CLIMB_HOLD, COURSE_EASE, COURSE_LEASH, COURSE_RATE, GLIDE_REST};

/// Returns the course a body eases back to with the keys let go, taken on the turn nearest
/// `course`.
///
/// Gliding it is the path of [`GLIDE_REST`]. Under power it is the course itself when it
/// climbs no steeper than [`CLIMB_HOLD`], so a climb set with the keys stays set, and level
/// when it dives: powered flight never holds a dive by itself. A key let go part way round a
/// loop eases the course to whichever of these is nearest.
pub(super) fn rest(state: &FlightState, course: f32) -> f32 {
    let glide = -(1.0 / GLIDE_REST).atan();
    let near = |angle: f32| course + wrap(angle - course);
    match state.mode {
        Mode::Fly => near(wrap(course).clamp(0.0, CLIMB_HOLD)),
        _ => near(glide),
    }
}

/// Returns the course after `h` seconds of the pitch stick `stick`.
///
/// A held stick turns it at [`COURSE_RATE`], never more than [`COURSE_LEASH`] ahead of the
/// flight frame: the frame only follows as fast as the wings turn the path, and a course that
/// ran a half turn ahead would call the nose back the short way. A course already farther
/// ahead is left where it is. Let go, a course that led the frame away from rest comes back to
/// the frame at once and eases on to rest from there, so the body does not go on diving or
/// climbing after the key. With the wings folded the course follows the frame, so that
/// spreading them starts from where the body points.
pub(super) fn steer(state: &FlightState, stick: f32, h: f32) -> f32 {
    if !state.mode.winged() {
        return state.pitch;
    }
    let stick = stick.clamp(-1.0, 1.0);
    let was = state.servo.course - state.pitch;
    let led = was + stick * COURSE_RATE * h;
    let held = state.pitch + led.clamp(was.min(-COURSE_LEASH), was.max(COURSE_LEASH));
    let rest = rest(state, held);
    let from = held.clamp(state.pitch.min(rest), state.pitch.max(rest));
    let back = from + (rest - from) * ease(COURSE_EASE, h);
    held + (back - held) * (1.0 - stick.abs())
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

/// Returns the unit direction the course of `state` points along, on the nose heading.
pub(super) fn ahead(state: &FlightState) -> Vec3 {
    Quat::from_euler(EulerRot::YXZ, state.servo.yaw, state.servo.course, 0.0) * Vec3::NEG_Z
}
