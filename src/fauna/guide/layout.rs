//! The measures of the guide: the room around it, its header and footer, the list on the left,
//! the card on the right, and how many rows of the list a window holds.

use bevy::math::Vec2;

use super::card::{Card, Row};
use crate::ui::theme;

/// Room in pixels between the window edge and the guide.
pub const EDGE: f32 = theme::GAP_L;

/// Width in pixels of the list.
pub const LIST: f32 = 320.0;

/// Room in pixels between the list and the card.
pub const BETWEEN: f32 = theme::GAP_L;

/// Widest card in pixels.
pub const CARD: f32 = 600.0;

/// Height in pixels of one row of the list.
pub const ROW: f32 = theme::ROW_COMPACT;

/// Height in pixels of a line of text, as a share of its font size.
const LEADING: f32 = 1.2;

/// Height in pixels of a line of small text.
pub const SMALL: f32 = theme::FONT_SMALL * LEADING;

/// Height in pixels of the header: the title, the count line and the room under them.
pub const HEAD: f32 = theme::FONT_HEADING * LEADING + SMALL + theme::GAP_M;

/// Height in pixels of the footer: the key hints and the room over them.
pub const FOOT: f32 = theme::GAP_S + SMALL;

/// The measures of the guide in one window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measures {
    /// Rows of the list drawn.
    pub rows: usize,
    /// Height in pixels left for the list and the card.
    pub room: f32,
    /// Width in pixels of the card.
    pub card: f32,
}

/// Returns the size in logical pixels of a window of `window` pixels at the interface scale
/// `scale`: at the usual size a 960 by 540 window is laid out as 1280 by 720.
pub fn logical(window: Vec2, scale: f32) -> Vec2 {
    window / scale
}

/// Returns the measures of the guide in a window of `window` logical pixels.
pub fn measure(window: Vec2) -> Measures {
    let room = (window.y - 2.0 * EDGE - HEAD - FOOT).max(ROW);
    let card = (window.x - 2.0 * EDGE - LIST - BETWEEN).clamp(0.0, CARD);
    Measures {
        rows: (room / ROW).floor() as usize,
        room,
        card,
    }
}

/// Returns the height in pixels of `card` once drawn: its title, its subtitle and its rows, a
/// heading after the first keeping [`theme::GAP_S`] over it.
pub fn height(card: &Card) -> f32 {
    let top = theme::FONT_BODY * LEADING + SMALL + theme::GAP_S;
    let rows = card.rows.iter().enumerate().map(|(at, row)| match row {
        Row::Heading(_) if at > 0 => SMALL + theme::GAP_S,
        _ => SMALL,
    });
    top + rows.sum::<f32>()
}

/// Returns how many characters of small text fit in `width` pixels.
pub fn chars(width: f32) -> usize {
    (width / (theme::FONT_SMALL * theme::GLYPH_ASPECT)) as usize
}
