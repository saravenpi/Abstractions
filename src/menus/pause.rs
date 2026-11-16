//! Pause menu: freezes physics while open.

use avian3d::prelude::*;
use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::pause_hud::{saved_label, show_saved, world_corner};
use super::{Menu, Nav};
use crate::saves::ActiveWorld;
use crate::screens::Screen;
use crate::ui::{Tone, caps, column, footer, item, root, theme};
use crate::world::WorldSeed;

/// The pause menu entries.
fn entries() -> impl Bundle {
    (
        column(px(theme::MENU_WIDTH), 0.0),
        children![
            (item("Resume", Tone::Plain), Nav::Resume, AutoFocus),
            (item("Settings", Tone::Plain), Nav::Settings),
            (item("Save and quit to menu", Tone::Plain), Nav::ToTitle),
            (item("Quit", Tone::Plain), Nav::Quit),
        ],
    )
}

/// Returns the corner text: the world name and seed, or the seed alone.
fn corner_text(active: Option<&ActiveWorld>, seed: Option<&WorldSeed>) -> String {
    match (active, seed) {
        (Some(active), _) => format!("{} - seed {}", active.0.save.name, active.0.save.seed),
        (None, Some(seed)) => format!("Seed {}", seed.0),
        (None, None) => String::new(),
    }
}

/// Spawns the pause menu with the world name and seed in the corner.
fn spawn(mut commands: Commands, seed: Option<Res<WorldSeed>>, active: Option<Res<ActiveWorld>>) {
    let corner = corner_text(active.as_deref(), seed.as_deref());
    commands.spawn((
        DespawnOnExit(Menu::Pause),
        root(),
        children![
            (
                caps("Paused"),
                Node {
                    margin: UiRect::bottom(px(theme::SPACE * 2.0)),
                    ..default()
                }
            ),
            entries(),
            footer("Esc to resume"),
            world_corner(corner),
            saved_label(),
        ],
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
        .add_systems(
            Update,
            (close_on_escape, show_saved).run_if(in_state(Menu::Pause)),
        );
}
