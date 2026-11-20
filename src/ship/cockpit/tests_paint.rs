//! Paint tests: the pixel font, the canvases and their bytes.

use super::font::{ADVANCE, HEIGHT, WIDTH, glyph};
use super::paint::{TEXT, block, write};
use super::pixels::Pixels;
use super::screens::Screen;

/// Returns how many pixels the glyph of `c` lights.
fn weight(c: char) -> u32 {
    glyph(c).iter().map(|row| row.count_ones()).sum()
}

/// Every digit, capital and unit letter the displays write has a glyph of its own, and a
/// character the font does not know is blank.
#[test]
fn the_font_has_every_character_of_the_displays() {
    let wanted = "0123456789.-/mkMlyAUSPEDHIGTWRNFCBOL";
    for c in wanted.chars() {
        assert!(weight(c) > 0, "{c} has no glyph");
    }
    let digits: Vec<[u8; HEIGHT]> = ('0'..='9').map(glyph).collect();
    for (i, a) in digits.iter().enumerate() {
        assert!(
            digits[i + 1..].iter().all(|b| a != b),
            "digit {i} is not unique"
        );
    }
    assert_eq!((weight(' '), weight('\u{1}')), (0, 0));
    assert!(
        wanted
            .chars()
            .all(|c| glyph(c).iter().all(|row| usize::from(*row) < 1 << WIDTH))
    );
}

/// A written text lights the pixels of its glyphs and nothing else, and the longest value the
/// displays write fits a block.
#[test]
fn a_text_is_painted_in_the_font() {
    let paint = |words: &str| {
        let mut pixels = Pixels::dark(TEXT.0, TEXT.1);
        write(&mut pixels, words, (0, 0));
        pixels
    };
    assert_eq!(paint("").count(), 0);
    assert_eq!(paint("10").count() as u32, weight('1') + weight('0'));
    let widest = "12.3 km/s";
    assert_eq!(
        paint(widest).count() as u32,
        widest.chars().map(weight).sum::<u32>()
    );
    assert!(widest.len() * ADVANCE <= TEXT.0);
    let rows = HEIGHT as i32;
    assert!(paint("HEIGHT").is_lit(0, 2) || (0..rows).any(|y| paint("HEIGHT").is_lit(1, y)));
}

/// A block with nothing to show is dark, and one with a reading lights its label, its figure
/// and its gauge.
#[test]
fn a_block_is_dark_without_a_source() {
    let lamps = ["AAA", "BBB", "CCC"];
    assert_eq!(
        block("SPEED", None, None, (&[("THR", None)], lamps)).count(),
        0
    );
    let lit = block(
        "SPEED",
        Some("10 m/s"),
        Some(16),
        (&[("THR", Some(8))], lamps),
    );
    assert_eq!((lit.width, lit.height), TEXT);
    assert!(lit.count() > 150);
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
