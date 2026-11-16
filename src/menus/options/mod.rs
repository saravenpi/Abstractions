//! Settings menu with graphics, controls and audio tabs.

mod actions;
mod field;
mod format;
mod hint;
mod pages;
mod rebind;
mod rows;
#[cfg(test)]
mod tests;
mod view;

use bevy::prelude::*;

use super::Menu;

pub(super) use rebind::capturing;

/// A section of the settings menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Graphics,
    Controls,
    Audio,
}

/// The section currently shown.
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
    app.add_systems(OnExit(Menu::Settings), reset_tab);
}

/// Returns to the first tab for the next visit.
fn reset_tab(mut tab: ResMut<SettingsTab>) {
    tab.0 = Tab::default();
}
