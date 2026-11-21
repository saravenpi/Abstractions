//! Settings menu with graphics, controls, ship, audio and general tabs, and a controls reference.

mod actions;
mod binds;
mod describe;
mod field;
mod format;
mod hint;
mod icons;
mod lights;
mod pages;
mod rebind;
mod reference;
mod rows;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_binds;
#[cfg(test)]
mod tests_lights;
#[cfg(test)]
mod tests_size;
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
    General,
    Help,
}

impl Tab {
    /// Every tab, in display order.
    pub const ALL: [Tab; 6] = [
        Tab::Graphics,
        Tab::Controls,
        Tab::Ship,
        Tab::Audio,
        Tab::General,
        Tab::Help,
    ];

    /// Returns the icon and the name shown on the tab. The icon pack has no spacecraft, so the
    /// ship tab wears the paper plane that also marks the ship on the compass.
    pub fn face(self) -> (Option<Icon>, &'static str) {
        match self {
            Tab::Graphics => (Some(Icon::Monitor), "Graphics"),
            Tab::Controls => (Some(Icon::Keyboard), "Controls"),
            Tab::Ship => (Some(Icon::ShipParked), "Ship"),
            Tab::Audio => (Some(Icon::Volume), "Audio"),
            Tab::General => (Some(Icon::Settings), "General"),
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
