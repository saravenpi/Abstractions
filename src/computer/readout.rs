//! The pane beside the table: the map, and under it the readout of the row under the cursor.

use bevy::prelude::*;

use super::board::{Board, VISIBLE_ROWS};
use super::detail::readout;
use super::map::field;
use crate::ui::{text, theme};

/// Width of the pane in pixels: the room of one readout line.
pub(super) const WIDTH: f32 = 300.0;

/// Height of the pane in pixels: the height of a full table, so the pane never makes the
/// screen taller than a long table already does.
pub(super) const HEIGHT: f32 = theme::ROW_COMPACT * (VISIBLE_ROWS + 1) as f32;

/// Height of one line of readout text in pixels: the small font at Bevy's line height of 1.2.
pub(super) const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Marks the text that shows the readout.
#[derive(Component)]
pub(super) struct Readout;

/// The map with the empty readout under it. `write` fills the readout.
pub(super) fn pane() -> impl Bundle {
    (
        Node {
            width: px(WIDTH),
            height: px(HEIGHT),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(theme::GAP_XS),
            overflow: Overflow::clip(),
            ..default()
        },
        children![field(), (Readout, text("", theme::FONT_SMALL, theme::TEXT))],
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
