//! Paint tests: the pixel font, the canvases and their bytes.

use super::font::{WIDTH, glyph};
use super::paint::{TEXT, text};
use super::pixels::Pixels;
use super::screens::Screen;

/// Returns how many pixels the glyph of `c` lights.
fn weight(c: char) -> u32 {
    glyph(c).iter().map(|row| row.count_ones()).sum()
}

/// Every digit and every unit letter the HUD writes has a glyph of its own, and a character
/// the font does not know is blank.
#[test]
fn the_font_has_every_character_of_the_units() {
    let wanted = "0123456789.-/mkMglys";
    for c in wanted.chars() {
        assert!(weight(c) > 0, "{c} has no glyph");
    }
    let digits: Vec<[u8; 5]> = ('0'..='9').map(glyph).collect();
    for (i, a) in digits.iter().enumerate() {
        assert!(
            digits[i + 1..].iter().all(|b| a != b),
            "digit {i} is not unique"
        );
    }
    assert_eq!((weight(' '), weight('?')), (0, 0));
    assert!(
        wanted
            .chars()
            .all(|c| glyph(c).iter().all(|row| usize::from(*row) < 1 << WIDTH))
    );
}

/// A text lights the pixels of its glyphs and nothing else, one column of margin on the left,
/// and the longest unit the HUD writes fits the canvas.
#[test]
fn a_text_is_painted_in_the_font() {
    assert_eq!(text("").count(), 0);
    assert_eq!(text("10").count() as u32, weight('1') + weight('0'));
    let long = text("999 gly/s");
    assert_eq!((long.width, long.height), TEXT);
    assert!(!(0..7).any(|y| long.is_lit(0, y)));
    let widest = text("1.23 gly/s");
    assert!(widest.is_lit(TEXT.0 as i32 - 1, 1));
    assert_eq!(
        widest.count() as u32,
        "1.23 gly/s".chars().map(weight).sum::<u32>()
    );
}

/// The bytes of a canvas are four per pixel, lit white over dark, and a pixel outside the
/// canvas is dropped.
#[test]
fn a_canvas_makes_its_bytes() {
    let mut pixels = Pixels::dark(2, 2);
    pixels.light(1, 0);
    pixels.light(5, 5);
    pixels.light(-1, 0);
    let bytes = pixels.rgba();
    assert_eq!(bytes.len(), 16);
    assert_eq!(&bytes[4..8], &[255, 255, 255, 255]);
    assert!(bytes[0] < 20 && bytes[8] < 20 && bytes[12] < 20);
    for screen in Screen::ALL {
        let (width, height) = screen.size();
        let dark = Pixels::dark(width, height);
        assert_eq!(dark.rgba().len(), width * height * 4);
    }
}
