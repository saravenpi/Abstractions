//! The look of the rows of the list: the selected row is the white block the keyboard and the
//! clicks move, and the row under the pointer, or whose mark the pointer is on, is lit faintly.
//! Hovering never moves the selection.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

use super::board::Board;
use super::chart::Dot;
use super::pointer::MapView;
use super::view::Slot;
use crate::ui::{Look, Selected, theme};

/// Returns the look of a row and whether it is lit: the selected row is the current one, black
/// on white, and a hovered row that is not selected is lit.
pub fn look(selected: bool, hovered: bool) -> (Look, bool) {
    match selected {
        true => (Look::Active, false),
        false => (Look::Rest, hovered),
    }
}

/// Returns the background of a row with `look`, lit or not: a white block under the selected
/// row, a faint one under a lit row, nothing otherwise.
pub fn surface(look: Look, lit: bool) -> Color {
    match (look, lit) {
        (Look::Active, _) => theme::BLOCK,
        (_, true) => theme::SELECT,
        _ => Color::NONE,
    }
}

/// What the look of one row is written to.
type Lamp = (
    &'static Slot,
    &'static RelativeCursorPosition,
    &'static mut Look,
    &'static mut Selected,
    &'static mut BackgroundColor,
);

/// Lights the drawn rows: the selected one, and the one under the pointer in the list or on
/// the map. It only writes what changed.
pub(super) fn light(board: Res<Board>, view: Res<MapView>, mut rows: Query<Lamp>) {
    let pointed = match view.hover {
        Some(Dot::Row(index)) => Some(index),
        _ => None,
    };
    for (slot, place, mut shown, mut lit, mut back) in &mut rows {
        let row = board.top + slot.0;
        let hovered = place.cursor_over || pointed == Some(row);
        let (wanted, glow) = look(row == board.cursor, hovered);
        shown.set_if_neq(wanted);
        lit.set_if_neq(Selected(glow));
        back.set_if_neq(BackgroundColor(surface(wanted, glow)));
    }
}
