//! Title screen: the game name with the main menu on top.

use bevy::prelude::*;

use super::Screen;
use crate::menus::Menu;
use crate::ui::{heading, theme};

/// Shows the game title and opens the main menu.
fn enter(mut commands: Commands, mut menu: ResMut<NextState<Menu>>) {
    commands.spawn((
        DespawnOnExit(Screen::Title),
        Node {
            position_type: PositionType::Absolute,
            top: percent(10),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![heading("Abstractions", theme::FONT_TITLE)],
    ));
    menu.set(Menu::Main);
}

/// Registers the title screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter);
}
