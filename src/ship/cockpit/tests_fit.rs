//! Tests that the flight display shows each legend with its value at every width of the value:
//! the legend in the small font on its own row, the value under it in the font of the game,
//! never cut, from a standstill to the speed of a long jump.

use super::faces::{FIGURE, FIGURES, FLIGHT, flight};
use super::font::{ADVANCE, SMALL_ADVANCE, SMALL_HEIGHT};
use super::paint::{MARGIN, key, words};
use super::pixels::Ink;

/// Returns true when the legend of the figure whose top is row `top` is painted.
fn legend(pixels: &super::pixels::Pixels, top: i32) -> bool {
    let rows = top + FIGURE.0..top + FIGURE.0 + SMALL_HEIGHT as i32;
    let columns = MARGIN..MARGIN + (3 * SMALL_ADVANCE) as i32;
    rows.into_iter()
        .any(|y| columns.clone().any(|x| pixels.ink(x, y) == Ink::Dim))
}

/// From 0 to billions of light-years, every speed and every height has its legend shown over it,
/// and the widest value fits between the margins of the display.
#[test]
fn every_value_shows_with_its_legend() {
    let room = FLIGHT.0 - 2 * MARGIN as usize;
    let mut widest = 0;
    for exponent in 0..26 {
        for mantissa in [1.0, 1.234_5, 5.5, 9.876_5] {
            let value = mantissa * 10f64.powi(exponent);
            let (speed, height) = (words(key(value), true), words(key(value), false));
            widest = widest.max(speed.chars().count() * ADVANCE);
            let shown = flight(
                [
                    (Some(speed.as_str()), Some(8)),
                    (Some(height.as_str()), Some(8)),
                ],
                None,
                [None; 2],
            );
            assert!(legend(&shown, FIGURES[0]), "no speed legend over {speed}");
            assert!(legend(&shown, FIGURES[1]), "no height legend over {height}");
        }
    }
    assert!(widest <= room + 1, "a value of {widest} pixels in {room}");
}
