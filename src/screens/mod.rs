//! Full-page game states: title, loading and gameplay.

mod gameplay;
mod loading;
mod title;

use bevy::prelude::*;

use crate::menus::Menu;

/// The logical screen the game is showing.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    /// Title screen with the main menu on top.
    #[default]
    Title,
    /// World generation around the spawn point.
    Loading,
    /// The player is on the islands.
    Gameplay,
}

/// Run condition: true while the player is in the world and no menu is open.
pub fn playing(screen: Res<State<Screen>>, menu: Res<State<Menu>>) -> bool {
    *screen.get() == Screen::Gameplay && *menu.get() == Menu::None
}

/// Registers the screen state and every screen plugin.
pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();
    app.add_plugins((title::plugin, loading::plugin, gameplay::plugin));
}
