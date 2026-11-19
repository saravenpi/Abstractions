//! The search box of the side panel and the pool of results under it.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;

use super::board::VISIBLE_ROWS;
use super::layout::{GLYPH, ROW};
use super::view::{Cell, Part, SEARCH_HINT, SearchBox, SearchText, Slot, Summary};
use crate::ui::{
    Icon, Look, Selected, Tone, cell, column, frame, hairline, icon, label, text, theme,
};

/// Most characters of the name of a result; a longer name is cut short.
pub(super) const TITLE_CHARS: usize = 19;

/// Most characters of the distance of a result, as `1000000 gly`.
pub(super) const DISTANCE_CHARS: usize = 11;

/// Widths in pixels of the icon, name and distance columns of a result. The selected result's
/// facts show in the card under the list.
pub(super) const WIDTHS: [f32; 3] = [
    theme::ICON,
    TITLE_CHARS as f32 * GLYPH,
    DISTANCE_CHARS as f32 * GLYPH,
];

/// One empty result of the pool, a button for the pointer only whose look `hover::light` sets.
/// `show::rows` writes its texts.
fn row(slot: usize) -> impl Bundle {
    let small =
        move |part: Part, tone: Tone| (label("", theme::FONT_SMALL, tone), Cell(slot, part));
    let mark = (icon(Icon::Check, Tone::Muted), Cell(slot, Part::Mark));
    let strip = Node {
        width: percent(100),
        height: px(ROW),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_XS),
        overflow: Overflow::clip(),
        ..default()
    };
    let button = (Button, RelativeCursorPosition::default(), Look::default());
    (
        (button, strip, BackgroundColor(Color::NONE)),
        children![
            cell(WIDTHS[0], mark),
            cell(WIDTHS[1], small(Part::Title, Tone::Plain)),
            cell(WIDTHS[2], small(Part::Distance, Tone::Plain)),
        ],
        (Slot(slot), Selected(false)),
    )
}

/// The search box: a framed button with the words typed, or the hint.
fn field() -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(theme::ROW_HEIGHT),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        border: UiRect::all(px(theme::LINE)),
        ..default()
    };
    let words = label(SEARCH_HINT, theme::FONT_SMALL, Tone::Muted);
    let inside = children![(SearchText, words)];
    (SearchBox, frame(), node, hairline(), inside)
}

/// The search box over its results: the box, the line that counts them and the pool of rows.
pub(super) fn search() -> impl Bundle {
    let rows = (
        column(percent(100), 0.0),
        Children::spawn(SpawnIter((0..VISIBLE_ROWS).map(row))),
    );
    let title = (
        Summary,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    (
        column(percent(100), theme::GAP_XS),
        children![field(), title, rows],
    )
}
