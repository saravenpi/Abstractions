//! The card of the panel: the readout of the row under the cursor.

use bevy::prelude::*;

use super::board::Board;
use super::detail::readout;
use super::layout::CARD;
use crate::ui::{text, theme};

/// Marks the text that shows the readout.
#[derive(Component)]
pub(super) struct Readout;

/// The empty card under a rule: the room of the longest readout, cut past it. `write` fills it.
pub(super) fn card() -> impl Bundle {
    (
        Node {
            height: px(CARD),
            flex_shrink: 0.0,
            padding: UiRect::top(px(theme::GAP_S)),
            border: UiRect::top(px(theme::LINE)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(theme::HAIRLINE),
        children![(Readout, text("", theme::FONT_SMALL, theme::TEXT))],
    )
}

/// Returns the readout of the row under the cursor, one fact per line.
pub(super) fn shown(board: &Board) -> String {
    let world = board.worlds.get(board.cursor).and_then(Option::as_ref);
    let line = board.lines.get(board.cursor);
    line.map_or_else(String::new, |line| readout(line, world).join("\n"))
}

/// Writes the readout when the table or the cursor changed, and when the screen was just opened.
pub(super) fn write(
    board: Res<Board>,
    fresh: Query<(), Added<Readout>>,
    mut texts: Query<&mut Text, With<Readout>>,
) {
    if !(board.is_changed() || !fresh.is_empty()) {
        return;
    }
    let wanted = shown(&board);
    for mut text in &mut texts {
        text.set_if_neq(Text(wanted.clone()));
    }
}
