//! The measures of the pack screen: the field guide's frame with a wider list, for the silhouette,
//! name, signature, mass and wear of each row.

use bevy::math::Vec2;

use crate::fauna::guide::layout::{BETWEEN, CARD, EDGE, chars, measure};

/// Width in pixels of the list.
pub const LIST: f32 = 440.0;

/// Characters of a name in a row.
pub const NAME: usize = 18;

/// Characters of a signature in a row.
pub const SIGNATURE: usize = 14;

/// Width in pixels of the wear bar of a row.
pub const BAR: f32 = 28.0;

/// Returns the rows of the list, the width of the card and the characters across it in a window
/// of `window` logical pixels.
pub fn fit(window: Vec2) -> (usize, f32, usize) {
    let rows = measure(window).rows;
    let card = (window.x - 2.0 * EDGE - LIST - BETWEEN).clamp(0.0, CARD);
    (rows, card, chars(card))
}
