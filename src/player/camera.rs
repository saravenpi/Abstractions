//! Mouse look and cursor grabbing.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use super::Player;
use super::controller::Motion;
use super::upright::pitch_of;
use crate::computer::Computer;
use crate::render::GameCamera;
use crate::screens::playing;
use crate::settings::{Persistent, Settings};
use crate::space::on_foot;

/// Radians of rotation per mouse pixel at sensitivity 1.0.
const RADIANS_PER_PIXEL: f32 = 0.0022;

/// Largest look-up or look-down angle in radians (89 degrees).
const PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;

/// Returns the new yaw and pitch after a mouse movement, with pitch clamped.
fn look_angles(yaw: f32, pitch: f32, delta: Vec2, scale: f32, invert_y: bool) -> (f32, f32) {
    let y_sign = if invert_y { -1.0 } else { 1.0 };
    let new_yaw = yaw - delta.x * scale;
    let new_pitch = (pitch - delta.y * scale * y_sign).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    (new_yaw, new_pitch)
}

/// Turns the heading around the local up with horizontal mouse motion and tilts the camera with
/// vertical motion. The controller turns the body to the heading in the same frame.
pub(super) fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    settings: Res<Persistent<Settings>>,
    mut heading: Single<&mut Motion, With<Player>>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    if motion.delta == Vec2::ZERO {
        return;
    }
    let controls = &settings.controls;
    let (yaw, pitch) = (heading.yaw, pitch_of(&camera));
    let scale = RADIANS_PER_PIXEL * controls.mouse_sensitivity;
    let (yaw, pitch) = look_angles(yaw, pitch, motion.delta, scale, controls.invert_y);
    heading.yaw = yaw;
    camera.rotation = Quat::from_rotation_x(pitch);
}

/// Locks and hides the cursor, or releases and shows it.
fn set_cursor(cursor: &mut CursorOptions, grabbed: bool) {
    let mode = if grabbed {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    if cursor.grab_mode != mode || cursor.visible == grabbed {
        cursor.grab_mode = mode;
        cursor.visible = !grabbed;
    }
}

/// Grabs the cursor while playing, unless a screen that needs the pointer is open.
fn grab_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor(&mut cursor, true);
}

/// Releases the cursor everywhere else, and while the board computer is open.
fn release_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor(&mut cursor, false);
}

/// Registers mouse look and cursor systems.
pub(super) fn plugin(app: &mut App) {
    let pointing = in_state(Computer::Open);
    app.add_systems(
        Update,
        (
            mouse_look.run_if(playing).run_if(on_foot),
            grab_cursor.run_if(playing.and_then(not(pointing.clone()))),
            release_cursor.run_if(not(playing).or_else(pointing)),
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pitch never passes the limit and inversion flips its sign.
    #[test]
    fn pitch_is_clamped_and_invertible() {
        let (_, pitch) = look_angles(0.0, 0.0, Vec2::new(0.0, -100000.0), 1.0, false);
        assert!((pitch - PITCH_LIMIT).abs() < 1e-5);
        let (_, normal) = look_angles(0.0, 0.0, Vec2::new(0.0, 10.0), 0.01, false);
        let (_, inverted) = look_angles(0.0, 0.0, Vec2::new(0.0, 10.0), 0.01, true);
        assert!(normal < 0.0 && inverted > 0.0);
    }
}
