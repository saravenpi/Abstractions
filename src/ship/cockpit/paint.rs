//! The strokes the displays of the dashboard are painted with: words in the pixel font, a bar,
//! and the steps a value is shown in, so a display repaints only when its picture changes.

use super::font::{ADVANCE, HEIGHT, WIDTH, glyph};
use super::pixels::{Ink, Pixels};
use super::tune::TEXT_STEPS;
use crate::computer::distance;

/// Steps of a bar from empty to full.
pub const BAR_STEPS: u8 = 32;

/// Pixels a display keeps free along its left and its right edge.
pub const MARGIN: i32 = 2;

/// Returns the step of the text display that shows `value`: its natural logarithm in steps of
/// half a percent. A value under zero reads as zero.
pub fn key(value: f64) -> i32 {
    ((1.0 + value.max(0.0)).ln() * TEXT_STEPS).round() as i32
}

/// Returns the value in the middle of the step `key`.
pub fn value(key: i32) -> f64 {
    (f64::from(key) / TEXT_STEPS).exp() - 1.0
}

/// Returns the words of the step `key`: its value in the units of the HUD, with the light-year
/// shortened to fit the display, and per second for a speed.
pub fn words(key: i32, per_second: bool) -> String {
    let unit = if per_second { "/s" } else { "" };
    format!(
        "{}{unit}",
        distance(value(key)).replace("light-years", "ly")
    )
}

/// Returns the column a text of `count` characters starts at to end against the right margin
/// of `pixels`.
pub fn flush(pixels: &Pixels, count: usize) -> i32 {
    pixels.width as i32 - MARGIN - (count * ADVANCE) as i32 + 1
}

/// Paints `words` in the pixel font in `ink`, with its top left corner at column `x` and row `y`.
pub fn write(pixels: &mut Pixels, words: &str, (x, y): (i32, i32), ink: Ink) {
    for (place, c) in words.chars().enumerate() {
        for (row, bits) in glyph(c).into_iter().enumerate().take(HEIGHT) {
            for column in (0..WIDTH).filter(|column| (bits >> (WIDTH - 1 - column)) & 1 == 1) {
                pixels.put(x + (place * ADVANCE + column) as i32, y + row as i32, ink);
            }
        }
    }
}

/// Paints a bar between the margins of the display, `rows` high from row `y`, lit up to `step`
/// of [`BAR_STEPS`] and dotted in grey beyond, so that the whole run shows.
pub fn bar(pixels: &mut Pixels, (y, rows): (i32, i32), step: u8) {
    let run = pixels.width as i32 - 2 * MARGIN;
    let lit = run * i32::from(step.min(BAR_STEPS)) / i32::from(BAR_STEPS);
    for column in 0..run {
        for row in 0..rows {
            if column < lit {
                pixels.put(MARGIN + column, y + row, Ink::Lit);
            } else if (column + row) % 2 == 0 {
                pixels.put(MARGIN + column, y + row, Ink::Dim);
            }
        }
    }
}
