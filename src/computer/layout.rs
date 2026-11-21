//! The measures of the map screen: the side panel on the right, and the square the chart
//! is laid out on in the middle of what is left. Every measure is in logical pixels at the 1280
//! by 720 reference: the interface scales with the window, so a window of 960 by 540 lays out
//! as 1280 by 720.
//!
//! The chart is drawn on a square, so its marks keep their shape. The square is as large as
//! the room beside the panel allows in its shorter direction, and the chart runs on past it
//! to the edges of that room when it is zoomed in.

use bevy::math::{Rect, Vec2};

use crate::ui::theme;

/// The smallest layout the game draws, in logical pixels: a window of 960 by 540 pixels scales
/// by 0.75 and lays out like this.
pub const SMALLEST: Vec2 = Vec2::new(1280.0, 720.0);

/// Width in pixels of the side panel: a line of small text holds 35 characters.
pub const SIDE: f32 = 384.0;

/// Margin in pixels between the texts over the map and the edges of the screen.
pub const MARGIN: f32 = theme::EDGE;

/// Padding in pixels of the side panel on every side.
pub const PAD: f32 = theme::S4;

/// Width in pixels the side panel leaves for its content, inside its padding.
pub const ROOM: f32 = SIDE - 2.0 * PAD;

/// Width in pixels of one character of small text, the font being monospaced.
pub const GLYPH: f32 = theme::GLYPH_ASPECT * theme::FONT_SMALL;

/// Most characters of small text one line of the side panel holds.
pub const LINE_CHARS: usize = (ROOM / GLYPH) as usize;

/// Most characters of the line that counts the results, at the right end of the heading of the
/// search, after the six spaced capitals of the heading and a gap.
pub const SUMMARY_CHARS: usize =
    ((ROOM - 6.0 * (GLYPH + theme::CAPS_SPACING) - theme::GAP_S) / GLYPH) as usize;

/// Most characters of the name of a place on the card, written in the body font.
pub const NAME_CHARS: usize = (ROOM / (theme::GLYPH_ASPECT * theme::FONT_BODY)) as usize;

/// Height in pixels of one row of the lists of the side panel: the results, the pins
/// and the players.
pub const ROW: f32 = theme::ROW_HEIGHT;

/// Height in pixels of one row of the pins and the players.
pub const LIST_ROW: f32 = theme::ROW_COMPACT;

/// Pixels between two sections of the side panel.
pub const SECTION_GAP: f32 = theme::S4;

/// Height in pixels of the heading of a section in spaced capitals, with the room under it.
pub const HEADING: f32 = LINE_HEIGHT + theme::GAP_S;

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

/// Rows of buttons the card keeps room for: the one the primary button is on.
pub const DEED_ROWS: usize = 1;

/// Lines the sentence under the primary button of the card may take.
pub const SENTENCE_LINES: usize = 2;

/// Height in pixels of the top of the card, which is never cut: its heading, the name, the
/// subtitle, the row of the primary button and the sentence under it, with the room between
/// them.
pub const CARD_HEAD: f32 = HEADING
    + theme::FONT_BODY * 1.2
    + LINE_HEIGHT
    + theme::GAP_S
    + DEED_ROWS as f32 * theme::ROW_HEIGHT
    + theme::GAP_XS
    + SENTENCE_LINES as f32 * LINE_HEIGHT
    + theme::GAP_S;

/// Height in pixels of the facts of the card at their longest: every row at the height of a
/// stat, with the room over each heading of a group but the first.
pub const FACTS: f32 =
    super::detail::ROWS as f32 * theme::LINE_SMALL + (super::detail::GROUPS - 1) as f32 * theme::S3;

/// Height in pixels of the foot of the pins: a line that says what the selected pin does, and
/// under it the row of its three buttons. With no pin selected the same room holds the empty
/// text, which takes two lines.
pub const PIN_FOOT: f32 = LINE_HEIGHT + theme::GAP_XS + LIST_ROW;

/// What the legend at the foot of the panel says: the keys of the map, in the order of the
/// table of keys.
pub const LEGEND: &str = "Click select  G go  B pin  H home  / search  Esc close";

/// Lines the legend takes at most.
pub const LEGEND_LINES: usize = 2;

/// Returns the height in pixels the side panel keeps for what is not the search results,
/// with `pins` and `players` rows listed: the padding, the gaps between the sections, the
/// search with its heading and box, the card at its tallest, the pins under their heading
/// with their foot, the players with their heading, which hide while there are none, and the
/// legend. In a window too short for all of it the facts at the bottom of the card are cut.
pub fn fixed((pins, players): (usize, usize)) -> f32 {
    let search = HEADING + theme::ROW_HEIGHT + theme::GAP_XS;
    let kept = HEADING + LIST_ROW * pins as f32 + PIN_FOOT;
    let friends = match players {
        0 => 0.0,
        count => SECTION_GAP + HEADING + count as f32 * LIST_ROW,
    };
    let legend = LEGEND_LINES as f32 * LINE_HEIGHT;
    2.0 * PAD + 3.0 * SECTION_GAP + search + CARD_HEAD + FACTS + kept + friends + legend
}

/// The measures of the screen in one window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Measures {
    /// Width of the side panel in pixels.
    pub side: f32,
    /// Side of the square map in pixels.
    pub map: f32,
    /// Rows of search results drawn.
    pub rows: usize,
    /// Rows of pins and of players drawn.
    pub lists: (usize, usize),
}

/// Returns how many of `lists` rows of pins and players a window `height` pixels high draws:
/// what is left once the search with its fewest results and the top of the card fit, the pins
/// first. The players show only while one of them fits under their heading.
pub fn listed(height: f32, (pins, players): (usize, usize)) -> (usize, usize) {
    let room = height - (fixed((0, 0)) - FACTS) - RESULTS.0 as f32 * ROW;
    let fits = |room: f32| (room / LIST_ROW).floor().max(0.0) as usize;
    let saved = pins.min(fits(room));
    let left = room - saved as f32 * LIST_ROW - SECTION_GAP - HEADING;
    (saved, players.min(fits(left)))
}

/// Returns the measures of the screen in a window of `window` logical pixels with `lists` rows
/// of pins and players: the panel keeps its width, the map takes the largest square
/// beside it, the lists take what the window has room for, and the search results take the
/// height the rest of the panel leaves, within their fewest and most rows.
pub fn measure(window: Vec2, lists: (usize, usize)) -> Measures {
    let room = Vec2::new(window.x - SIDE, window.y);
    let lists = listed(window.y, lists);
    let rows = ((window.y - fixed(lists)) / ROW).floor().max(0.0) as usize;
    Measures {
        side: SIDE,
        map: room.min_element().max(0.0).floor(),
        rows: rows.clamp(RESULTS.0, RESULTS.1),
        lists,
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
