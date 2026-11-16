//! Shared UI camera, theme and widgets.

mod camera;
mod focus;
mod fps;
mod item;
mod style;
pub mod theme;
mod widgets;

pub use focus::move_focus;
pub use item::{item, item_with, label, link, row_with, step_button};
pub use style::{Look, Tone};
pub use widgets::{caps, column, footer, hairline, root, root_on, rule, text};

use bevy::prelude::*;

/// Registers the UI camera and widget systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, (camera::spawn_camera, fps::spawn))
        .add_systems(Update, fps::update)
        .add_systems(Update, camera::sync_output)
        .add_systems(
            Update,
            (
                style::focus_on_hover,
                style::update_looks,
                style::paint_texts,
            )
                .chain(),
        );
}
