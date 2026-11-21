//! The header over the map: the path of the levels with the Home control beside it, and the
//! caption under them.

use bevy::prelude::*;

use super::course::Act;
use crate::ui::{Icon, KEY_GAP, Tip, Tone, entry, frame, theme};

/// The Home control: a click flies the ship home, as Fly home on the card of the home planet
/// does with the Go key.
fn home() -> impl Bundle {
    let node = Node {
        height: px(theme::ROW_COMPACT),
        align_items: AlignItems::Center,
        column_gap: px(KEY_GAP),
        padding: UiRect::horizontal(px(theme::S2)),
        ..default()
    };
    (
        frame(),
        Act::Home,
        Tip("Fly home"),
        node,
        children![entry(Icon::Home, "Home", theme::FONT_SMALL, Tone::Plain)],
    )
}

/// Returns the first line of the header: the path of the levels, then the Home control.
pub(super) fn top() -> impl Bundle {
    let row = Node {
        align_items: AlignItems::Center,
        column_gap: px(theme::S4),
        ..default()
    };
    (row, children![crate::map::path(), home()])
}
