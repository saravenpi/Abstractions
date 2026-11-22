//! Menu overlays: main, worlds, new world, multiplayer, settings, update, pause and players, and
//! the in-game HUD.

mod hud;
mod main_menu;
mod multiplayer;
mod name;
mod nav;
mod new_game;
mod options;
mod pause;
mod pause_hud;
mod players;
mod recall;
mod seed;
mod slots;
#[cfg(test)]
mod tests;
mod update;
mod worlds;

pub use hud::{HudWeather, Marker, MarkerKind, MarkerPlace, MarkerSet, Markers};
pub use nav::{Nav, Trail};
pub use options::{SettingsTab, Tab as SettingsPage};

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
    /// A newer release: its notes, its download and the restart.
    Update,
    /// Saved servers and games on the local network.
    Multiplayer,
    /// Waiting for a server to answer a join.
    Joining,
    /// The players of the session, opened from the pause menu.
    Players,
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
        update::plugin,
        multiplayer::plugin,
        players::plugin,
    ));
}
