//! Menu overlays: main, worlds, new world, settings and pause, and the in-game HUD.

mod hud;
mod main_menu;
mod name;
mod nav;
mod new_game;
mod options;
mod pause;
mod pause_hud;
mod recall;
mod seed;
mod slots;
#[cfg(test)]
mod tests;
mod worlds;

pub use nav::{Nav, Trail};
pub use options::key_name;

use bevy::prelude::*;

/// The menu overlay currently open, if any.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Menu {
    /// No menu is open.
    #[default]
    None,
    /// Main menu on the title screen.
    Main,
    /// Name and seed selection before a new world.
    NewGame,
    /// List of saved worlds.
    Worlds,
    /// Graphics, controls and audio settings.
    Settings,
    /// Pause menu during gameplay.
    Pause,
}

/// Registers the menu state and every menu plugin.
pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>()
        .init_resource::<recall::LastEntry>()
        .add_observer(recall::remember);
    app.add_plugins((
        nav::plugin,
        hud::plugin,
        main_menu::plugin,
        worlds::plugin,
        new_game::plugin,
        options::plugin,
        pause::plugin,
    ));
}
