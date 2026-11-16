//! Shared UI camera, theme and widgets.

mod style;
pub mod theme;
mod widgets;

pub use widgets::{button, button_with, heading, label, panel, root, row};

use bevy::prelude::*;

/// Spawns the transparent overlay camera that draws all UI above the 3D view.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
}

/// Registers the UI camera and widget systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(Update, style::paint_buttons);
}
