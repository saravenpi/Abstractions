//! Settings menu with graphics, controls, ship and audio tabs, and a controls reference.

mod actions;
mod binds;
mod field;
mod format;
mod hint;
mod pages;
mod rebind;
mod reference;
mod rows;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_binds;
#[cfg(test)]
mod tests_width;
mod view;

use bevy::prelude::*;

#[cfg(test)]
pub(super) use actions::SettingsAction;
#[cfg(test)]
pub(super) use field::Field;
pub use format::key_name;
pub(super) use rebind::capturing;

use crate::ui::Icon;

/// A section of the settings menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Graphics,
    Controls,
    Ship,
    Audio,
    Help,
}

impl Tab {
    /// Every tab, in display order.
    pub const ALL: [Tab; 5] = [
        Tab::Graphics,
        Tab::Controls,
        Tab::Ship,
        Tab::Audio,
        Tab::Help,
    ];

    /// Returns the icon and the name shown on the tab. The icon pack has no spacecraft, so the
    /// ship tab has a label only.
    pub fn face(self) -> (Option<Icon>, &'static str) {
        match self {
            Tab::Graphics => (Some(Icon::Monitor), "Graphics"),
            Tab::Controls => (Some(Icon::Keyboard), "Controls"),
            Tab::Ship => (None, "Ship"),
            Tab::Audio => (Some(Icon::Volume), "Audio"),
            Tab::Help => (Some(Icon::Info), "Reference"),
        }
    }
}

/// The section currently shown. It is kept between visits.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SettingsTab(pub Tab);

/// Registers the settings menu systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SettingsTab>().add_plugins((
        view::plugin,
        actions::plugin,
        rebind::plugin,
        hint::plugin,
    ));
}
