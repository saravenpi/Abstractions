//! The measures of the map screen: the slim side panel on the right, and the square the chart
//! is laid out on in the middle of what is left.
//!
//! The chart is drawn on a square, so its marks keep their shape. The square is as large as
//! the room beside the panel allows in its shorter direction, and the chart runs on past it
//! to the edges of that room when it is zoomed in.

use bevy::math::Vec2;

use crate::ui::theme;

/// The smallest window the game supports, in pixels.
pub const SMALLEST: Vec2 = Vec2::new(960.0, 540.0);

/// Width in pixels of the side panel, its border included.
pub const SIDE: f32 = 320.0;

/// Margin in pixels around the side panel and around the texts over the map.
pub const MARGIN: f32 = 12.0;

/// Height in pixels of one row of the lists of the side panel.
pub const ROW: f32 = theme::ROW_COMPACT;

/// Fewest and most rows of search results drawn.
pub const RESULTS: (usize, usize) = (3, 8);

/// Height of one line of small text in pixels: the small font at Bevy's line height of 1.2.
pub const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Height in pixels the side panel keeps for what is not the search results: the search box,
/// the card of the selected result, the favourites and the players with their headings, the
/// hints and the margins.
pub const FIXED: f32 = 470.0;

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

/// Returns the measures of the screen in a window of `window` pixels: the panel keeps its
/// width, the map takes the largest square beside it, and the search results take the height
/// the rest of the panel leaves, within their fewest and most rows.
pub fn measure(window: Vec2) -> Measures {
    let room = Vec2::new(window.x - SIDE, window.y);
    let rows = ((window.y - FIXED) / ROW).floor().max(0.0) as usize;
    Measures {
        side: SIDE,
        map: room.min_element().max(0.0).floor(),
        rows: rows.clamp(RESULTS.0, RESULTS.1),
    }
}
