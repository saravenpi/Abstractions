//! Paint tests: the pixel font, the strokes, the canvases and their bytes.

use super::faces::{FLIGHT, SIDE, drive, flight, status};
use super::font::{ADVANCE, HEIGHT, WIDTH, glyph};
use super::paint::{bar, flush, write};
use super::pixels::{Ink, Pixels};
use super::readings::Lamp;
use super::screens::Screen;
use super::tune::{BAYS, TEXEL};

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

/// A written text paints the pixels of its glyphs and nothing else, in the ink it is given, and
/// the longest value the displays write fits the flight display.
#[test]
fn a_text_is_painted_in_the_font() {
    let paint = |words: &str| {
        let mut pixels = Pixels::dark(FLIGHT.0, FLIGHT.1);
        write(&mut pixels, words, (0, 0), Ink::Lit);
        pixels
    };
    assert_eq!(paint("").count(), 0);
    assert_eq!(paint("10").count() as u32, weight('1') + weight('0'));
    let widest = "123.4 km/s";
    assert_eq!(
        paint(widest).count() as u32,
        widest.chars().map(weight).sum::<u32>()
    );
    assert!(flush(&paint(""), widest.len()) >= 2);
    let mut amber = Pixels::dark(8, 9);
    write(&mut amber, "H", (0, 0), Ink::Amber);
    assert_eq!(amber.ink(0, 0), Ink::Amber);
    assert!(HEIGHT * 2 + 4 <= 34 && ADVANCE == WIDTH + 1);
}

/// A bar is lit up to its step and dotted in grey beyond, between the margins of the display.
#[test]
fn a_bar_shows_its_whole_run() {
    let mut pixels = Pixels::dark(SIDE.0, SIDE.1);
    bar(&mut pixels, (10, 4), 16);
    assert_eq!(pixels.ink(2, 10), Ink::Lit);
    assert_eq!(pixels.ink(15, 13), Ink::Lit);
    assert!(pixels.ink(28, 10) == Ink::Dim || pixels.ink(29, 10) == Ink::Dim);
    assert!(!pixels.is_lit(0, 10) && !pixels.is_lit(31, 10) && !pixels.is_lit(10, 9));
}

/// A display with nothing to show is dark, and one with a reading shows it: the drive its
/// bars, the flight its figures, the status its lamps in their colours.
#[test]
fn a_display_is_dark_without_a_source() {
    assert_eq!(drive([None; 3]).count(), 0);
    assert_eq!(flight([(None, None); 2], None, [None; 2]).count(), 0);
    let driven = drive([Some(16), None, Some(32)]);
    assert!(driven.count() > 150 && !driven.is_lit(2, 30));
    let flown = flight([(Some("10 m/s"), Some(8)), (None, None)], None, [None; 2]);
    assert!(flown.count() > 60 && flown.ink(2, 3) == Ink::Dim);
    let out = status([false; 6]);
    assert!(out.count() > 100 && (0..72).all(|y| (0..32).all(|x| out.ink(x, y) != Ink::Lit)));
    let hot = status(Lamp::ALL.map(|lamp| lamp == Lamp::Hot || lamp == Lamp::Failing));
    assert_eq!(hot.ink(5, 4 + 11 * 3 + 3), Ink::Amber);
    assert_eq!(hot.ink(5, 4 + 11 * 5 + 3), Ink::Red);
    assert_eq!(hot.ink(5, 4 + 3), Ink::Dark);
}

/// The bytes of a canvas are four per pixel, lit white over dark, a pixel outside the canvas
/// is dropped, and every display is as large as its bay on the panel.
#[test]
fn a_canvas_makes_its_bytes() {
    let mut pixels = Pixels::dark(2, 2);
    pixels.put(1, 0, Ink::Lit);
    pixels.put(5, 5, Ink::Lit);
    pixels.put(-1, 0, Ink::Lit);
    let bytes = pixels.rgba();
    assert_eq!(bytes.len(), 16);
    assert_eq!(&bytes[4..8], &[255, 255, 255, 255]);
    assert!(bytes[0] < 20 && bytes[8] < 20 && bytes[12] < 20);
    for screen in Screen::ALL {
        let (width, height) = screen.size();
        let dark = Pixels::dark(width, height);
        assert_eq!(dark.rgba().len(), width * height * 4);
        let bay = BAYS[screen as usize];
        assert!((width as f32 * TEXEL - bay.1).abs() < 1.0e-5);
        assert!((height as f32 * TEXEL - BAYS[1].1).abs() < 1.0e-5);
    }
}
