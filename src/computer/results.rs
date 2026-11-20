//! The search box of the side panel and the pool of results under it.

use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;

use super::board::VISIBLE_ROWS;
use super::layout::{GLYPH, ROW};
use super::section::{heading, heading_row, row_node};
use super::view::{Cell, Part, SEARCH_HINT, SearchBox, SearchText, Slot, Summary};
use crate::ui::{Icon, Look, Selected, Tone, cell, column, frame, icon, label, text, theme};

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
    let strip = row_node(ROW);
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

/// The search box: a button with the words typed, or the hint.
fn field() -> impl Bundle {
    let node = Node {
        width: percent(100),
        height: px(theme::ROW_HEIGHT),
        align_items: AlignItems::Center,
        column_gap: px(theme::GAP_S),
        padding: UiRect::horizontal(px(theme::GAP_S)),
        ..default()
    };
    let words = label(SEARCH_HINT, theme::FONT_SMALL, Tone::Muted);
    let inside = children![(SearchText, words)];
    (SearchBox, frame(), node, inside)
}

/// The heading of the search, with the line that names the results and counts them at its
/// right end, muted.
fn head() -> impl Bundle {
    let count = (
        Summary,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    );
    (heading_row(), children![heading("Search"), count])
}

/// The search section: its heading with the count, the box and the pool of rows under it.
pub(super) fn search() -> impl Bundle {
    let rows = (
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            margin: UiRect::top(px(theme::GAP_XS)),
            ..default()
        },
        Children::spawn(SpawnIter((0..VISIBLE_ROWS).map(row))),
    );
    (column(percent(100), 0.0), children![head(), field(), rows])
}
