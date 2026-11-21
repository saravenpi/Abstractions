//! Players menu: who shares the session, opened from the pause menu while joined.

use bevy::input_focus::AutoFocus;
use bevy::prelude::*;

use super::{Menu, Nav};
use crate::net::Players;
use crate::ui::{
    Cancels, Icon, Scroller, Tone, entry, footer, heading, hints, item, panel, root, theme,
};

/// Grid columns of the names and of Back: room for a name of the most characters a name may
/// have, and for the line that says the player is alone, with an icon (320 pixels of the 368).
const COLUMNS: u32 = 5;

/// Returns the lines of the menu: one name each, or a line saying the player is alone.
pub(super) fn lines(players: &Players) -> Vec<String> {
    match players.0.is_empty() {
        true => vec!["Nobody else is here yet".to_string()],
        false => players.0.iter().map(|(_, name)| name.clone()).collect(),
    }
}

/// The column of the names: it gives up its height first when the window is too short for every
/// name, and its names scroll, so the heading and Back stay in view.
fn names() -> impl Bundle {
    let node = Node {
        width: px(theme::span(COLUMNS)),
        flex_direction: FlexDirection::Column,
        row_gap: px(theme::GAP_S),
        min_height: px(0),
        overflow: Overflow::scroll_y(),
        ..default()
    };
    (Scroller, ScrollPosition::default(), node)
}

/// Spawns the list of players and the Back entry, which takes the focus.
fn spawn(mut commands: Commands, players: Res<Players>) {
    let menu = commands
        .spawn((
            DespawnOnExit(Menu::Players),
            root(),
            children![footer(hints(&[("Back", "Esc")]))],
        ))
        .id();
    let card = commands
        .spawn((panel(), ChildOf(menu), children![heading("Players")]))
        .id();
    let list = commands.spawn((names(), ChildOf(card))).id();
    let symbol = (!players.0.is_empty()).then_some(Icon::User);
    for line in lines(&players) {
        let shown = entry(symbol, &line, theme::FONT_BODY, Tone::Plain);
        commands.spawn((shown, ChildOf(list)));
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
        width: px(theme::span(COLUMNS)),
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
