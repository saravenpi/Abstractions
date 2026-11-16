//! Menu overlays: main, new game, settings and pause.

mod main_menu;
mod nav;
mod new_game;
mod options;
mod pause;
mod seed;

pub use nav::{Nav, SettingsOrigin};

use bevy::prelude::*;

/// The menu overlay currently open, if any.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Menu {
    /// No menu is open.
    #[default]
    None,
    /// Main menu on the title screen.
    Main,
    /// Seed selection before a new game.
    NewGame,
    /// Graphics, controls and audio settings.
    Settings,
    /// Pause menu during gameplay.
    Pause,
}

/// Registers the menu state and every menu plugin.
pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();
    app.add_plugins((
        nav::plugin,
        main_menu::plugin,
        new_game::plugin,
        options::plugin,
        pause::plugin,
    ));
}
