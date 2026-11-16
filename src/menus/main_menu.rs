//! Main menu shown over the title screen.

use bevy::prelude::*;

use super::{Menu, Nav};
use crate::ui::{button, panel, root, theme};

/// Spawns the main menu.
fn spawn(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(Menu::Main),
        root(),
        children![(
            panel(),
            children![
                (
                    button("New game", theme::BUTTON_WIDTH),
                    Nav::To(Menu::NewGame)
                ),
                (button("Settings", theme::BUTTON_WIDTH), Nav::Settings),
                (button("Quit", theme::BUTTON_WIDTH), Nav::Quit),
            ]
        )],
    ));
}

/// Registers the main menu systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn);
}
