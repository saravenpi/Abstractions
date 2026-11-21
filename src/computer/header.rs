//! The header over the map: the path of the levels with the Home control beside it, and the
//! caption under them.

use bevy::prelude::*;

use super::course::Act;
use super::keys::HOME_KEY;
use crate::ui::{Icon, Tip, Tone, entry, frame, key_label, theme};

/// The Home control: the way home with its key, which flies the ship home as the key does.
fn home() -> impl Bundle {
    let node = Node {
        height: px(theme::ROW_COMPACT),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        padding: UiRect::horizontal(px(theme::S2)),
        ..default()
    };
    (
        frame(),
        Act::Home,
        Tip("Fly home"),
        node,
        children![
            entry(Icon::Home, "Home", theme::FONT_SMALL, Tone::Plain),
            key_label(HOME_KEY, theme::FONT_SMALL, Tone::Muted)
        ],
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
