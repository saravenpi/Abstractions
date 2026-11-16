//! Title screen: a calm flat background and the version label under the main menu.

use bevy::prelude::*;

use super::Screen;
use crate::menus::Menu;
use crate::ui::{root_on, text, theme};

/// Spawns the flat background and opens the main menu.
fn enter(mut commands: Commands, mut menu: ResMut<NextState<Menu>>) {
    commands.spawn((
        DespawnOnExit(Screen::Title),
        root_on(theme::BACKGROUND, JustifyContent::FlexStart),
        children![(
            text(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                theme::FONT_SMALL,
                theme::MUTED
            ),
            Node {
                position_type: PositionType::Absolute,
                right: px(theme::EDGE),
                bottom: px(theme::EDGE),
                ..default()
            }
        )],
    ));
    menu.set(Menu::Main);
}

/// Registers the title screen systems.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter);
}
