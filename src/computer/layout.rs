//! The measures of the side panel: its margins, how wide it is, how large its map is and how
//! many rows of the list fit, for a window of a given size.
//!
//! The panel is a column of five sections: the header, the square map, the list, the card of
//! the selected row and the footer with the main button. The same margin sits around the panel
//! and between its sections, and grows with the window. Everything but the map and the list
//! has a fixed height. The map takes most of what is left, and the panel is as wide as the map
//! and its margins, so nothing stands empty beside it, unless the card needs more room.

use bevy::math::Vec2;

use super::board::VISIBLE_ROWS;
use super::detail::LINES;
use crate::ui::theme;

/// The smallest window the game supports, in pixels.
pub const SMALLEST: Vec2 = Vec2::new(960.0, 540.0);

/// Share of the window width the map and its margins may take.
pub const PANEL_SHARE: f32 = 0.4;

/// Narrowest and widest room in pixels inside the margins of the panel. The narrowest holds
/// the longest line of the card.
pub const CONTENT: (f32, f32) = (396.0, 864.0);

/// Margin in pixels around the panel and between its sections, in a window 1080 pixels high.
pub const MARGIN_AT_1080: f32 = 24.0;

/// Smallest and largest margin in pixels, whatever the window.
pub const MARGIN: (f32, f32) = (12.0, 48.0);

/// Share of the height left by the fixed sections that the map takes.
pub const MAP_SHARE: f32 = 0.65;

/// Fewest rows of the list drawn, however short the window is.
pub const MIN_ROWS: usize = 2;

/// Height in pixels of one row of the list.
pub const ROW: f32 = theme::ROW_HEIGHT;

/// Height in pixels of the main button.
pub const BUTTON: f32 = theme::ROW_HEIGHT + theme::GAP_S;

/// Height of one line of small text in pixels: the small font at Bevy's line height of 1.2.
pub const LINE_HEIGHT: f32 = theme::FONT_SMALL * 1.2;

/// Height in pixels of the header: the breadcrumb and the tabs.
pub const HEADER: f32 = LINE_HEIGHT + theme::GAP_XS + theme::ROW_HEIGHT;

/// Height in pixels of the title of the list, the line that counts its rows, with its gap.
pub const TITLE: f32 = LINE_HEIGHT + theme::GAP_XS;

/// Height in pixels of the card: its rule, its margin, the lines of the longest readout and the
/// climate line under them.
pub const CARD: f32 = theme::LINE + theme::GAP_S + (LINES + 1) as f32 * LINE_HEIGHT;

/// Height in pixels of the footer: the course line, the main button and the key hints.
pub const FOOTER: f32 = 2.0 * LINE_HEIGHT + BUTTON + 2.0 * theme::GAP_XS;

/// Height in pixels of everything that does not grow: the header, the title of the list, the
/// card and the footer.
pub const FIXED: f32 = HEADER + TITLE + CARD + FOOTER;

/// Margins down the panel: above the first section, between the five and under the last.
pub const MARGINS: f32 = 6.0;

/// The measures of the panel in one window.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Measures {
    /// Pixels around the panel and between its sections.
    pub margin: f32,
    /// Width of the panel in pixels, margins included.
    pub panel: f32,
    /// Side of the square map in pixels.
    pub map: f32,
    /// Rows of the list drawn.
    pub rows: usize,
}

/// Returns the height in pixels of everything but the map and the list, with `margin` pixels
/// around and between the sections.
pub fn chrome(margin: f32) -> f32 {
    FIXED + MARGINS * margin
}

/// Returns the margin in a window `height` pixels high, in whole pixels.
pub fn margin(height: f32) -> f32 {
    (height * MARGIN_AT_1080 / 1080.0)
        .round()
        .clamp(MARGIN.0, MARGIN.1)
}

/// Returns the measures of the panel in a window of `window` pixels: the map takes its share
/// of the height the fixed sections leave, as long as the panel may be that wide and the list
/// keeps its fewest rows, and the list takes the rest.
pub fn measure(window: Vec2) -> Measures {
    let margin = margin(window.y);
    let room = window.y - chrome(margin);
    let widest = (window.x * PANEL_SHARE - 2.0 * margin).clamp(CONTENT.0, CONTENT.1);
    let tall = room - MIN_ROWS as f32 * ROW;
    let map = (room * MAP_SHARE).min(tall).min(widest).max(0.0).floor();
    let rows = ((room - map) / ROW).floor().max(0.0) as usize;
    Measures {
        margin,
        panel: map.max(CONTENT.0) + 2.0 * margin,
        map,
        rows: rows.clamp(MIN_ROWS, VISIBLE_ROWS),
    }
}
