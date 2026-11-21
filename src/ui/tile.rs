//! Pointer tiles: a card that can be chosen, and a tall row of two lines. Both are buttons with no
//! keyboard focus: the screen that holds them moves its own cursor and marks the chosen one with
//! [`Selected`], which tints it with `SELECT`. The pointer tints the one under it with `HOVER`.
//!
//! ```text
//! card      [ picture | words ............ | numbers ]   padding S3, blocks S3 apart
//! row_tall  [ [i] name ......................... mass ]   ROW_TALL high, padding INSET, any mark
//!           [     fact .................. value  bar  ]
//! ```

use bevy::prelude::*;

use super::item::frame;
use super::style::Selected;
use super::theme;

/// Marks a card.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Tile;

/// Marks a tall row.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct TallRow;

/// A card the pointer can choose: blocks side by side, [`theme::S3`] apart and from its edge,
/// tinted while `chosen`. Its children are its blocks; on a narrow card the last ones wrap to a
/// line of their own, [`theme::S3`] under the others.
pub fn card(chosen: bool) -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_shrink: 0.0,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::FlexStart,
        column_gap: px(theme::S3),
        row_gap: px(theme::S3),
        padding: UiRect::all(px(theme::S3)),
        ..default()
    };
    (Tile, frame(), Selected(chosen), node)
}

/// One line of words and marks inside a row or a card: left to right, [`theme::S2`] apart,
/// centred on the line. A child that sets `flex_grow` pushes the next ones to the right end.
pub fn line() -> Node {
    Node {
        width: percent(100),
        align_items: AlignItems::Center,
        column_gap: px(theme::S2),
        min_height: px(theme::LINE_SMALL),
        ..default()
    }
}

/// A row of two lines, [`theme::ROW_TALL`] high: `mark` on the icon line, an [`super::icon_slot`] or any
/// picture of the same [`theme::ICON`] size, then `top` over `under`, each built on a [`line`].
/// Tinted while `chosen`.
pub fn row_tall(
    mark: impl Bundle,
    chosen: bool,
    top: impl Bundle,
    under: impl Bundle,
) -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(theme::ROW_TALL),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::ICON_GAP),
        padding: UiRect::horizontal(px(theme::INSET)),
        ..default()
    };
    let words = Node {
        flex_grow: 1.0,
        min_width: px(0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::clip(),
        ..default()
    };
    (
        TallRow,
        frame(),
        Selected(chosen),
        node,
        children![mark, (words, children![top, under])],
    )
}
