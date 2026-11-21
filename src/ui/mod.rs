//! Shared UI camera, theme, fonts, icons, backdrop and widgets.

mod backdrop;
mod camera;
#[cfg(test)]
mod camera_tests;
mod chrome;
#[cfg(test)]
mod color_rig;
mod focus;
mod font;
#[cfg(test)]
mod font_rig;
mod fps;
mod icon;
#[cfg(test)]
mod icon_tests;
mod item;
mod keys;
mod list;
pub mod motion;
#[cfg(test)]
mod rules_tests;
mod scroll;
mod sound;
mod style;
mod table;
#[cfg(test)]
mod tests_panel;
pub mod theme;
#[cfg(test)]
mod theme_tests;
mod tooltip;
pub mod units;
mod widgets;

pub use backdrop::{Backdrop, Blurred, backdrop_luminance, contrast, warm_up};
pub use camera::{MIN_WIDTH, WorldView, ui_scale, ui_scale_for};
pub use chrome::{action, heading, notice, strip};
pub use focus::move_focus;
pub use icon::{ICON_FONT, ICONS_VERSION, Icon, icon, icon_sized, icon_slot};
pub use item::{entry, frame, item, item_with, label, link, step_button};
pub use keys::{
    HINT_GAP, HintLabel, KEY_GAP, KeyName, Legend, footer, hints, key_cap, key_hint, legend,
};
pub use list::Window;
pub use scroll::{BAR, BAR_GAP, ScrollArea, Scroller, Thumb, scroll_area, scroller, thumb};
pub use sound::Cancels;
pub use style::{Filled, Look, Selected, Tone};
pub use table::{cell, head, row_with, table_head, table_row, value_cell};
pub use tooltip::Tip;
pub use widgets::{Overlay, caps, column, panel, root, root_on, rule, text};

use bevy::input_focus::InputFocus;
use bevy::prelude::*;

use crate::settings::{Persistent, Settings};
use bevy::window::PrimaryWindow;

/// Registers the camera, the fonts and the overlays that draw above the game.
fn overlay(app: &mut App) {
    app.init_resource::<UiScale>()
        .add_systems(
            Startup,
            (
                camera::spawn_camera,
                fps::spawn,
                icon::load,
                font::load,
                tooltip::spawn,
            ),
        )
        .add_systems(Update, (fps::update, camera::sync_output, tooltip::update))
        .add_systems(Update, camera::frame_view.run_if(resized));
}

/// Registers the UI camera, the fonts, the backdrop, the widget looks and the interface sounds.
pub(super) fn plugin(app: &mut App) {
    overlay(app);
    app.add_plugins(backdrop::plugin)
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
                scroll::bars.run_if(any_with_component::<scroll::ScrollArea>),
                (sound::moved, scroll::follow_focus).run_if(resource_changed::<InputFocus>),
            ),
        );
}

/// Whether the interface must be sized again: the primary window changed, or the settings did.
fn resized(
    windows: Query<(), (With<PrimaryWindow>, Changed<bevy::window::Window>)>,
    settings: Option<Res<Persistent<Settings>>>,
) -> bool {
    !windows.is_empty() || settings.is_some_and(|settings| settings.is_changed())
}
