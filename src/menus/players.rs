//! Players menu: who shares the session, opened from the pause menu while joined.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::{Menu, Nav};
use crate::net::Players;
use crate::ui::{Cancels, Icon, Tone, column, entry, footer, heading, item, panel, root, theme};

/// Returns the lines of the menu: one name each, or a line saying the player is alone.
pub(super) fn lines(players: &Players) -> Vec<String> {
    match players.0.is_empty() {
        true => vec!["Nobody else is here yet".to_string()],
        false => players.0.iter().map(|(_, name)| name.clone()).collect(),
    }
}

/// Spawns the list of players and the Back entry, which takes the focus.
fn spawn(mut commands: Commands, players: Res<Players>) {
    let menu = commands
        .spawn((
            DespawnOnExit(Menu::Players),
            root(),
            children![footer("Esc goes back")],
        ))
        .id();
    let card = commands
        .spawn((panel(), ChildOf(menu), children![heading("Players")]))
        .id();
    let names = commands
        .spawn((column(px(theme::MENU_WIDTH), theme::GAP_S), ChildOf(card)))
        .id();
    let symbol = (!players.0.is_empty()).then_some(Icon::User);
    for line in lines(&players) {
        let shown = entry(symbol, &line, theme::FONT_BODY, Tone::Plain);
        commands.spawn((shown, ChildOf(names)));
    }
    commands.spawn((back(), ChildOf(card)));
}

/// The Back entry under the names, which takes the focus.
fn back() -> impl Bundle {
    let back = (
        item(Icon::Back, "Back", Tone::Plain),
        Nav::Back,
        Cancels,
        AutoFocus,
    );
    let below = Node {
        width: px(theme::MENU_WIDTH),
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(px(theme::GAP_L)),
        ..default()
    };
    (below, children![back])
}

/// Registers the players menu.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Players), spawn);
}
