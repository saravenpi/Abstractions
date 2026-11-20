//! Shared UI camera, theme, fonts, icons, backdrop and widgets.

mod backdrop;
mod camera;
mod chrome;
mod focus;
mod font;
mod fps;
mod icon;
#[cfg(test)]
mod icon_tests;
mod item;
mod list;
pub mod motion;
mod scroll;
mod sound;
mod style;
mod table;
pub mod theme;
#[cfg(test)]
mod theme_tests;
mod widgets;

pub use backdrop::{Backdrop, Blurred, backdrop_luminance, contrast, warm_up};
pub use chrome::{action, footer, heading, key_cap, notice, strip};
pub use focus::move_focus;
pub use icon::{ICON_FONT, ICONS_VERSION, Icon, icon, icon_sized, icon_slot};
pub use item::{entry, frame, item, item_with, label, link, step_button};
pub use list::Window;
pub use scroll::{Scroller, scroller};
pub use sound::Cancels;
pub use style::{Look, Selected, Tone};
pub use table::{cell, head, row_with, table_head, table_row, value_cell};
pub use widgets::{Overlay, caps, column, hairline, panel, root, root_on, rule, text};

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::render::window_changed;

/// Registers the UI camera, the fonts, the backdrop, the widget looks and the interface sounds.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins(backdrop::plugin)
        .add_systems(
            Startup,
            (camera::spawn_camera, fps::spawn, icon::load, font::load),
        )
        .add_systems(Update, (fps::update, camera::sync_output))
        .add_systems(Update, camera::frame_view.run_if(window_changed))
        .add_observer(sound::activated)
        .add_systems(
            Update,
            (
                style::focus_on_hover,
                style::update_looks,
                style::paint_texts,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                sound::escaped,
                scroll::wheel,
                (sound::moved, scroll::follow_focus).run_if(resource_changed::<InputFocus>),
            ),
        );
}
