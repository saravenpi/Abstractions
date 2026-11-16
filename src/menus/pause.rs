//! Pause menu: freezes physics while open.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::{Menu, Nav};
use crate::screens::Screen;
use crate::ui::{button, heading, panel, root, theme};

/// Spawns the pause menu.
fn spawn(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(Menu::Pause),
        root(),
        children![(
            panel(),
            children![
                heading("Paused", theme::FONT_HEADING),
                (button("Resume", theme::BUTTON_WIDTH), Nav::Resume),
                (button("Settings", theme::BUTTON_WIDTH), Nav::Settings),
                (button("Main menu", theme::BUTTON_WIDTH), Nav::ToTitle),
                (button("Quit", theme::BUTTON_WIDTH), Nav::Quit),
            ]
        )],
    ));
}

/// Freezes physics.
fn freeze(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

/// Resumes physics.
fn thaw(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

/// Closes the pause menu on Escape.
fn close_on_escape(keys: Res<ButtonInput<KeyCode>>, mut menu: ResMut<NextState<Menu>>) {
    if keys.just_pressed(KeyCode::Escape) {
        menu.set(Menu::None);
    }
}

/// Registers the pause menu systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), (spawn, freeze))
        .add_systems(OnEnter(Menu::None), thaw)
        .add_systems(OnExit(Screen::Gameplay), thaw)
        .add_systems(Update, close_on_escape.run_if(in_state(Menu::Pause)));
}
