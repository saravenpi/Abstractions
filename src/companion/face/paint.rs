//! Painting a face frame: thirty-two by sixteen pixels, each lit or dark.

use super::Frame;
use super::sprites::{Eye, eyes, mouth};

/// Width of the screen in pixels.
pub const WIDTH: usize = 32;

/// Height of the screen in pixels.
pub const HEIGHT: usize = 16;

/// Column of the centre of the left eye. The right eye mirrors it.
const EYE_X: i32 = 9;

/// Row of the centre of the eyes.
const EYE_Y: i32 = 6;

/// First row of the mouth, and its first column.
const MOUTH: (i32, i32) = (12, 12);

/// Pixels the eyes move sideways when the gaze shifts.
const GAZE_STEP: i32 = 2;

/// sRGB colour of a lit pixel and of a dark one. The material multiplies both by the face colour.
const INK: ([u8; 4], [u8; 4]) = ([255, 255, 255, 255], [9, 13, 20, 255]);

/// The painted pixels: one row per `u32`, the highest bit on the left of the screen.
pub type Canvas = [u32; HEIGHT];

/// Lights the pixels of `bits`, a row of `width` pixels whose first column is `x`, on row `y`.
///
/// Pixels outside the screen are dropped.
fn row(canvas: &mut Canvas, bits: u32, width: i32, (x, y): (i32, i32)) {
    if !(0..HEIGHT as i32).contains(&y) {
        return;
    }
    let shift = WIDTH as i32 - width - x;
    let placed = if shift >= 0 {
        u64::from(bits) << shift
    } else {
        u64::from(bits) >> -shift
    };
    canvas[y as usize] |= placed as u32;
}

/// Stamps an eye centred on column `x` and row `y`.
fn stamp(canvas: &mut Canvas, eye: Eye, (x, y): (i32, i32)) {
    for (line, bits) in eye.into_iter().enumerate() {
        row(canvas, u32::from(bits), 7, (x - 3, y - 3 + line as i32));
    }
}

/// Paints a frame: both eyes moved by the gaze, and the mouth following by half as much.
pub fn paint(frame: &Frame) -> Canvas {
    let mut canvas = [0; HEIGHT];
    let (left, right) = eyes(frame.expr, frame.lid, frame.beat);
    let (dx, dy) = (frame.gaze.x * GAZE_STEP, -frame.gaze.y);
    stamp(&mut canvas, left, (EYE_X + dx, EYE_Y + dy));
    stamp(
        &mut canvas,
        right,
        (WIDTH as i32 - 1 - EYE_X + dx, EYE_Y + dy),
    );
    for (line, bits) in mouth(frame.expr, frame.beat).into_iter().enumerate() {
        let at = (MOUTH.1 + frame.gaze.x, MOUTH.0 + line as i32);
        row(&mut canvas, u32::from(bits), 8, at);
    }
    canvas
}

/// Returns the number of lit pixels.
#[cfg(test)]
pub fn lit(canvas: &Canvas) -> u32 {
    canvas.iter().map(|row| row.count_ones()).sum()
}

/// Returns true when the left half of the picture mirrors the right half.
#[cfg(test)]
pub fn symmetric(canvas: &Canvas) -> bool {
    canvas.iter().all(|row| *row == row.reverse_bits())
}

/// Returns the picture as sRGB RGBA bytes, row by row from the top left.
pub fn rgba(canvas: &Canvas) -> Vec<u8> {
    let pixel = |row: u32, x: usize| match (row >> (WIDTH - 1 - x)) & 1 {
        1 => INK.0,
        _ => INK.1,
    };
    canvas
        .iter()
        .flat_map(|row| (0..WIDTH).flat_map(move |x| pixel(*row, x)))
        .collect()
}
