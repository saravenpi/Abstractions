//! Painting the displays of the dashboard: the text of a speed or a height, the attitude ball
//! and the scope of the course. Each painter is a pure function of the quantised value it
//! shows, so a display repaints only when that value changes.

use bevy::prelude::*;

use super::font::{WIDTH, glyph};
use super::pixels::Pixels;
use super::tune::{ANGLE_STEP, TEXT_STEPS};
use crate::computer::distance;

/// Size in pixels of the text displays: ten characters of four columns and a row of margin.
pub const TEXT: (usize, usize) = (40, 7);

/// Size in pixels of the attitude ball and of the scope, which are square with an odd side so
/// that they have a centre pixel.
pub const BALL: usize = 17;

/// Radius in pixels of the rim of the ball, and the pixels a radian of pitch moves the horizon.
const ROUND: (f32, f32) = (8.0, 6.0);

/// Radius in pixels of the area of the scope the marks move in.
pub const REACH: f32 = 6.0;

/// Returns the step of the text display that shows `value`: its natural logarithm in steps of
/// half a percent. A value under zero reads as zero.
pub fn key(value: f64) -> i32 {
    ((1.0 + value.max(0.0)).ln() * TEXT_STEPS).round() as i32
}

/// Returns the value in the middle of the step `key`.
pub fn value(key: i32) -> f64 {
    (f64::from(key) / TEXT_STEPS).exp() - 1.0
}

/// Returns the words of the step `key`: its value in the units of the HUD, and per second for
/// a speed.
pub fn words(key: i32, per_second: bool) -> String {
    let unit = if per_second { "/s" } else { "" };
    format!("{}{unit}", distance(value(key)))
}

/// Paints `words` in the pixel font, one column of margin on the left and one row above.
pub fn text(words: &str) -> Pixels {
    let mut pixels = Pixels::dark(TEXT.0, TEXT.1);
    for (place, c) in words.chars().enumerate() {
        for (row, bits) in glyph(c).into_iter().enumerate() {
            for column in (0..WIDTH).filter(|column| (bits >> (WIDTH - 1 - column)) & 1 == 1) {
                pixels.light((1 + place * (WIDTH + 1) + column) as i32, 1 + row as i32);
            }
        }
    }
    pixels
}

/// Paints the attitude ball for a pitch up and a roll to the right of `pitch` and `roll`
/// steps of the display: the ground is a lit dither under a lit horizon, the sky is dark, and
/// the rim and the nose of the ship are lit. The horizon turns the other way from the ship.
pub fn horizon(pitch: i32, roll: i32) -> Pixels {
    let mut pixels = Pixels::dark(BALL, BALL);
    let (sin, cos) = (roll as f32 * ANGLE_STEP).sin_cos();
    let lift = (pitch as f32 * ANGLE_STEP).clamp(-1.2, 1.2) * ROUND.1;
    let centre = (BALL / 2) as i32;
    for y in 0..BALL as i32 {
        for x in 0..BALL as i32 {
            let (px, py) = ((x - centre) as f32, (centre - y) as f32);
            let out = px.hypot(py);
            let above = -px * sin + py * cos + lift;
            let rim = out > ROUND.0 - 1.0 && out <= ROUND.0;
            let line = out <= ROUND.0 && above.abs() < 0.75;
            let ground = out <= ROUND.0 && above < 0.0 && (x + y) % 2 == 0;
            if rim || line || ground {
                pixels.light(x, y);
            }
        }
    }
    pixels.light(centre, centre);
    pixels
}

/// Lights a plus of five pixels around `at`, the mark of the course.
fn cross(pixels: &mut Pixels, at: IVec2) {
    for step in [IVec2::ZERO, IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
        pixels.light(at.x + step.x, at.y + step.y);
    }
}

/// Lights a ring of eight pixels around `at`, the mark of the body.
fn ring(pixels: &mut Pixels, at: IVec2) {
    for y in -1..=1 {
        for x in -1..=1 {
            if x != 0 || y != 0 {
                pixels.light(at.x + x, at.y + y);
            }
        }
    }
}

/// Paints the scope: the rim as dots, the nose at the centre, the course as a plus and the
/// body as a ring, each at its point from [`project`](super::attitude::project). Nothing is
/// lit when the scope has neither.
pub fn scope(course: Option<IVec2>, body: Option<IVec2>) -> Pixels {
    let mut pixels = Pixels::dark(BALL, BALL);
    if course.is_none() && body.is_none() {
        return pixels;
    }
    let centre = IVec2::splat((BALL / 2) as i32);
    for y in 0..BALL as i32 {
        for x in 0..BALL as i32 {
            let out = ((x - centre.x) as f32).hypot((y - centre.y) as f32);
            if out > ROUND.0 - 1.0 && out <= ROUND.0 && (x + y) % 2 == 0 {
                pixels.light(x, y);
            }
        }
    }
    pixels.light(centre.x, centre.y);
    let at = |mark: IVec2| centre + IVec2::new(mark.x, -mark.y);
    body.map(at)
        .into_iter()
        .for_each(|mark| ring(&mut pixels, mark));
    course
        .map(at)
        .into_iter()
        .for_each(|mark| cross(&mut pixels, mark));
    pixels
}
