//! The measures of the map screen: the slim side panel on the right, and the square the chart
//! is laid out on in the middle of what is left.
//!
//! The chart is drawn on a square, so its marks keep their shape. The square is as large as
//! the room beside the panel allows in its shorter direction, and the chart runs on past it
//! to the edges of that room when it is zoomed in.

use bevy::math::{Rect, Vec2};

use crate::ui::theme;

/// The smallest window the game supports, in pixels.
pub const SMALLEST: Vec2 = Vec2::new(960.0, 540.0);

/// Width in pixels of the side panel.
pub const SIDE: f32 = 320.0;

/// Margin in pixels around the side panel and around the texts over the map.
pub const MARGIN: f32 = theme::GAP_M;

/// Width in pixels the side panel leaves for its content, inside its margins.
pub const ROOM: f32 = SIDE - 2.0 * MARGIN;

/// Width in pixels of one character of small text, the font being monospaced.
pub const GLYPH: f32 = theme::GLYPH_ASPECT * theme::FONT_SMALL;

/// Most characters of small text one line of the side panel holds.
pub const LINE_CHARS: usize = (ROOM / GLYPH) as usize;

/// Height in pixels of one row of the lists of the side panel.
pub const ROW: f32 = theme::ROW_COMPACT;

/// Fewest and most rows of search results drawn.
pub const RESULTS: (usize, usize) = (1, 8);

/// Height of one line of small text in pixels: the small font at Bevy's line height of 1.2.
pub const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Height in pixels of the band along the top of the map that the header lines take: where
/// the path with the Back button, where the player is and the caption under it.
pub const HEAD_BAND: f32 = MARGIN + theme::ROW_COMPACT + 2.0 * LINE_HEIGHT + 2.0 * theme::GAP_XS;

/// Height in pixels of the band along the bottom of the map that the footer lines take: the
/// action line, the course line and the place under the pointer.
pub const FOOT_BAND: f32 = MARGIN + theme::ROW_COMPACT + 2.0 * LINE_HEIGHT + 2.0 * theme::GAP_XS;

/// Returns true when the box `label` of a name on a map `side` pixels high stays clear of the
/// header and footer bands, so it never runs under their words.
pub fn clear_of_bands(label: Rect, side: f32) -> bool {
    label.min.y >= HEAD_BAND && label.max.y <= side - FOOT_BAND
}

/// Height in pixels of the card of the selected result at its tallest.
const CARD: f32 = theme::GAP_S + super::detail::LINES as f32 * LINE_HEIGHT + theme::LINE;

/// Returns the height in pixels the side panel keeps for what is not the search results,
/// with `favourites` and `players` rows listed: the margins, the search box and the line over
/// the results, the card at its tallest, the favourites with their heading and button, and
/// the players with their heading, which hide while there are none.
pub fn fixed((favourites, players): (usize, usize)) -> f32 {
    let search = theme::ROW_HEIGHT + 2.0 * theme::GAP_XS + LINE_HEIGHT;
    let saved = LINE_HEIGHT + 2.0 * theme::GAP_XS + theme::ICON * (1 + favourites) as f32;
    let friends = match players {
        0 => 0.0,
        count => theme::GAP_M + LINE_HEIGHT + theme::GAP_XS + count as f32 * LINE_HEIGHT,
    };
    2.0 * MARGIN + 2.0 * theme::GAP_M + search + CARD + saved + friends
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
}

/// Returns the measures of the screen in a window of `window` pixels with `lists` rows of
/// favourites and players: the panel keeps its width, the map takes the largest square
/// beside it, and the search results take the height the rest of the panel leaves, within
/// their fewest and most rows.
pub fn measure(window: Vec2, lists: (usize, usize)) -> Measures {
    let room = Vec2::new(window.x - SIDE, window.y);
    let rows = ((window.y - fixed(lists)) / ROW).floor().max(0.0) as usize;
    Measures {
        side: SIDE,
        map: room.min_element().max(0.0).floor(),
        rows: rows.clamp(RESULTS.0, RESULTS.1),
    }
}
