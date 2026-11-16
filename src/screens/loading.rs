//! Loading screen shown while the world generates.

use bevy::prelude::*;

use super::Screen;
use crate::menus::Menu;
use crate::ui::{heading, root, theme};
use crate::world::WorldReady;

/// Shows the loading text.
fn enter(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(Screen::Loading),
        root(),
        children![heading("Generating islands...", theme::FONT_HEADING)],
    ));
}

/// Moves to gameplay once the world reports it is ready.
fn wait_for_world(
    ready: Option<Res<WorldReady>>,
    mut screen: ResMut<NextState<Screen>>,
    mut menu: ResMut<NextState<Menu>>,
) {
    if ready.is_some_and(|ready| ready.0) {
        screen.set(Screen::Gameplay);
        menu.set(Menu::None);
    }
}

/// Registers the loading screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter)
        .add_systems(Update, wait_for_world.run_if(in_state(Screen::Loading)));
}
