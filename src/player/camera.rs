//! Cursor grabbing. The look itself belongs to the camera rig in [`crate::view`].

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::computer::Computer;
use crate::screens::playing;

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

/// Registers the cursor systems.
pub(super) fn plugin(app: &mut App) {
    let pointing = in_state(Computer::Open);
    app.add_systems(
        Update,
        (
            grab_cursor.run_if(playing.and_then(not(pointing.clone()))),
            release_cursor.run_if(not(playing).or_else(pointing)),
        ),
    );
}
