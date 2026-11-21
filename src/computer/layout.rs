//! The measures of the map screen: the side panel on the right, whose facts and lists scroll, and the square the chart
//! is laid out on in the middle of what is left. Every measure is in logical pixels at the 1280
//! by 720 reference: the interface scales with the window, so a window of 960 by 540 lays out
//! as 1280 by 720.
//!
//! Both are panels inside the margins of the screen, so the blurred game shows around them.
//!
//! The chart is drawn on a square, so its marks keep their shape. The square is as large as
//! the panel of the map allows in its shorter direction, and the chart runs on past it to the
//! edges of that panel when it is zoomed in.

use bevy::math::{Rect, Vec2};

use crate::ui::theme;

/// The layout of the reference window, in logical pixels: a window of 960 by 540 pixels at the
/// usual interface size scales by 0.75 and lays out like this.
pub const SMALLEST: Vec2 = Vec2::new(1280.0, 720.0);

/// Width in pixels of the side panel: a line of small text holds 38 characters. At 1280 by 720
/// it is a third of the window and the square picture beside it is as large as the height
/// allows.
pub const SIDE: f32 = 416.0;

/// Margin in pixels between the two panels of the screen and the edges of the window.
pub const EDGE: f32 = theme::EDGE;

/// Pixels between the panel of the map and the side panel.
pub const BETWEEN: f32 = theme::S4;

/// Margin in pixels between the texts over the map and the edges of its panel: the padding of
/// a panel.
pub const MARGIN: f32 = theme::S4;

/// Padding in pixels of the side panel on every side.
pub const PAD: f32 = theme::S4;

/// Width in pixels the side panel leaves for its content, inside its padding.
pub const ROOM: f32 = SIDE - 2.0 * PAD;

/// Width in pixels of one character of small text, the font being monospaced.
pub const GLYPH: f32 = theme::GLYPH_ASPECT * theme::FONT_SMALL;

/// Most characters of small text one line of the side panel holds.
pub const LINE_CHARS: usize = (ROOM / GLYPH) as usize;

/// Most characters of the line that counts the results, at the right end of the heading of the
/// search, after its icon, the six spaced capitals of the heading and a gap.
pub const SUMMARY_CHARS: usize =
    ((ROOM - theme::ICON - theme::S2 - 6.0 * (GLYPH + theme::CAPS_SPACING) - theme::GAP_S) / GLYPH)
        as usize;

/// Most characters of the name of a place on the card, written in the body font after its icon.
pub const NAME_CHARS: usize =
    ((ROOM - theme::ICON - theme::S2) / (theme::GLYPH_ASPECT * theme::FONT_BODY)) as usize;

/// Height in pixels of one row of the lists of the side panel: the results, the pins
/// and the players.
pub const ROW: f32 = theme::ROW_HEIGHT;

/// Height in pixels of one row of the pins and the players.
pub const LIST_ROW: f32 = theme::ROW_COMPACT;

/// Pixels between two sections of the side panel.
pub const SECTION_GAP: f32 = theme::S3;

/// Height in pixels of the heading of a section, an icon and spaced capitals, with the room
/// under it.
pub const HEADING: f32 = theme::ICON + theme::GAP_S;

/// Fewest and most rows of search results drawn.
pub const RESULTS: (usize, usize) = (1, 8);

/// Height of one line of small text in pixels: the small font at Bevy's line height of 1.2.
pub const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Height in pixels of the band along the top of the map that the header lines take: the
/// path of levels and the caption under it.
pub const HEAD_BAND: f32 = MARGIN + theme::ROW_COMPACT + LINE_HEIGHT + theme::GAP_XS;

/// Height in pixels of the band along the bottom of the map that the footer lines take: the
/// progress of the route, the course line and the place under the pointer.
pub const FOOT_BAND: f32 = MARGIN + 3.0 * LINE_HEIGHT + 2.0 * theme::GAP_XS;

/// Returns true when the box `label` of a name on a map `side` pixels high stays clear of the
/// header and footer bands, so it never runs under their words.
pub fn clear_of_bands(label: Rect, side: f32) -> bool {
    label.min.y >= HEAD_BAND && label.max.y <= side - FOOT_BAND
}

/// Height in pixels of the Go button of the card: an icon with the padding of the block.
pub const GO: f32 = theme::ICON + 2.0 * theme::S2;

/// Lines the sentence under the primary button of the card may take.
pub const SENTENCE_LINES: usize = 2;

/// Height in pixels of the head of the card, which never scrolls: the row of the icon and the
/// name, the subtitle, the Go button and the sentence under it, with the room between them.
pub const CARD_HEAD: f32 =
    theme::ICON + LINE_HEIGHT + theme::GAP_S + GO + theme::S2 + SENTENCE_LINES as f32 * LINE_HEIGHT;

/// Height in pixels of the foot of the pins: a line that says what the selected pin does, and
/// under it the row of its three buttons. With no pin selected the same room holds the empty
/// text, which takes two lines.
pub const PIN_FOOT: f32 = LINE_HEIGHT + theme::GAP_XS + LIST_ROW;

/// The keys of the map at the foot of the panel: what each does, then the key. Go, Home and the
/// search show their key on their own control, so the legend does not say them again.
pub const LEGEND: [(&str, &str); 4] = [
    ("Select", "Click"),
    ("Pin", "B"),
    ("Zoom or scroll", "Wheel"),
    ("Close", "Esc"),
];

/// Lines the legend takes at most.
pub const LEGEND_LINES: usize = 2;

/// Height in pixels the legend takes at most: its lines with the room between them.
pub const LEGEND_HEIGHT: f32 =
    LEGEND_LINES as f32 * LINE_HEIGHT + (LEGEND_LINES - 1) as f32 * theme::S1;

/// Height in pixels the part of the panel that scrolls keeps before the results take more rows
/// than their fewest: nine rows of facts.
pub const BODY: f32 = 9.0 * theme::LINE_SMALL;

/// Height in pixels of what never scrolls in the side panel, the search results apart: the
/// margins of the screen over and under the panel, the padding, the gaps between the four parts,
/// the search with its heading and its box, the head of the card and the legend. The rest of the
/// height goes to the results and to the part that scrolls: the facts of the card, the pins and
/// the players.
pub const FIXED: f32 = 2.0 * EDGE
    + 2.0 * PAD
    + 3.0 * SECTION_GAP
    + HEADING
    + theme::ROW_HEIGHT
    + theme::GAP_XS
    + CARD_HEAD
    + LEGEND_HEIGHT;

/// The measures of the screen in one window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Measures {
    /// Width of the side panel in pixels.
    pub side: f32,
    /// Side of the square map in pixels.
    pub map: f32,
    /// Rows of search results drawn.
    pub rows: usize,
}

/// Returns the measures of the screen in a window of `window` logical pixels: both panels sit
/// inside the margins of the screen with a gap between them, the side panel keeps its width, the
/// map takes the largest square that fits the panel beside it, the search results take the rows
/// the height leaves once the part that scrolls has [`BODY`], within their fewest and most rows, and
/// the part that scrolls takes the rest: see [`FIXED`].
pub fn measure(window: Vec2) -> Measures {
    let room = Vec2::new(window.x - SIDE - BETWEEN, window.y) - 2.0 * EDGE;
    let rows = ((window.y - FIXED - BODY) / ROW).floor().max(0.0) as usize;
    let rows = rows.clamp(RESULTS.0, RESULTS.1);
    Measures {
        side: SIDE,
        map: room.min_element().max(0.0).floor(),
        rows,
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
