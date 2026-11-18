//! The measures of the side panel: how wide it is, how large its map is and how many rows of
//! the list fit, for a window of a given size.
//!
//! The panel is a column: the header, the square map, the list, the card of the selected row
//! and the footer. Everything but the map and the list has a fixed height, so the map and the
//! list share what the window leaves.

use bevy::math::Vec2;

use super::board::VISIBLE_ROWS;
use super::detail::LINES;
use crate::ui::theme;

/// The smallest window the game supports, in pixels.
pub const SMALLEST: Vec2 = Vec2::new(960.0, 540.0);

/// Share of the window width the panel takes.
pub const PANEL_SHARE: f32 = 0.4;

/// Narrowest and widest panel in pixels.
pub const PANEL_WIDTH: (f32, f32) = (420.0, 640.0);

/// Room in pixels between the edge of the panel and what it holds.
pub const PAD: f32 = theme::GAP_M;

/// Share of the window height the map takes when the window has room for it.
pub const MAP_SHARE: f32 = 0.34;

/// Fewest rows of the list drawn, however short the window is.
pub const MIN_ROWS: usize = 2;

/// Height of one line of small text in pixels: the small font at Bevy's line height of 1.2.
pub const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Height in pixels of the header: the breadcrumb, the tabs and the line that counts the rows.
pub const HEADER: f32 = 2.0 * LINE_HEIGHT + theme::ROW_HEIGHT + 2.0 * theme::GAP_XS;

/// Height in pixels of the card: its rule, its margin, the lines of the longest readout and the
/// climate line under them.
pub const CARD: f32 = theme::LINE + theme::GAP_S + (LINES + 1) as f32 * LINE_HEIGHT;

/// Height in pixels of the footer: the course line, the actions and the key hints.
pub const FOOTER: f32 = 2.0 * LINE_HEIGHT + theme::ROW_HEIGHT + 2.0 * theme::GAP_XS;

/// Height in pixels of everything but the map and the list, with the gaps between the five
/// blocks of the panel and the room around them.
pub const CHROME: f32 = 2.0 * PAD + HEADER + CARD + FOOTER + 4.0 * theme::GAP_S;

/// Returns the width of the panel in a window `window_width` pixels wide.
pub fn panel_width(window_width: f32) -> f32 {
    (window_width * PANEL_SHARE).clamp(PANEL_WIDTH.0, PANEL_WIDTH.1)
}

/// Returns the side of the square map in a window of `window` pixels: its share of the height,
/// unless the panel is too narrow or the list would lose its last rows.
pub fn map_side(window: Vec2) -> f32 {
    let wide = panel_width(window.x) - 2.0 * PAD;
    let tall = window.y - CHROME - MIN_ROWS as f32 * theme::ROW_COMPACT;
    (window.y * MAP_SHARE).min(wide).min(tall).max(0.0)
}

/// Returns how many rows of the list are drawn in a window of `window` pixels.
pub fn rows_that_fit(window: Vec2) -> usize {
    let room = window.y - CHROME - map_side(window);
    let rows = (room / theme::ROW_COMPACT).floor().max(0.0) as usize;
    rows.clamp(MIN_ROWS, VISIBLE_ROWS)
}
