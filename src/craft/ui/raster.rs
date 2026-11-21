//! Drawing the picture of a tool pixel by pixel, at the exact size it shows at. The tool lies on
//! the diagonal, its head at the top right as the tools of the icon font do, and each pixel is
//! either on or off: it is tested at its centre against the shapes, with no blend at the edges,
//! so the picture stays crisp at any size. The shaft is one stroke of the icon font wide, a
//! twelfth of the picture, and each part is drawn in the grey of its material. Over the live world
//! the HUD shadow is laid under it, as under every HUD text.

use std::f32::consts::FRAC_1_SQRT_2;

use bevy::color::ColorToPacked;

use crate::craft::build::{Figure, Look};
use crate::ui::theme;

/// Bytes of one pixel: red, green, blue and alpha.
pub const BYTES: usize = 4;

/// The tip of the head, in shares of the picture from its top left.
const TIP: (f32, f32) = (0.78, 0.22);

/// The butt of the longest tool, in shares of the picture.
const BUTT: (f32, f32) = (0.10, 0.90);

/// Share of the longest tool each length draws, shortest first.
const SPAN: [f32; 4] = [0.55, 0.7, 0.85, 1.0];

/// Where a pixel centre lies on a tool, in shares of the picture: `back`, the distance from the
/// tip back along the shaft, and `side`, the distance across it, positive toward the bottom right.
#[derive(Debug, Clone, Copy)]
struct Spot {
    /// From the tip back along the shaft.
    back: f32,
    /// Across the shaft.
    side: f32,
}

/// Returns whether `at` is on a leaf point `long` and at most `wide` on each side, sharp at the
/// tip and widest seven tenths of the way back.
fn leaf(at: Spot, long: f32, wide: f32) -> bool {
    let (t, v) = (at.back, at.side);
    (0.0..=long).contains(&t) && v.abs() <= wide * (t / (0.7 * long)).min((long - t) / (0.3 * long))
}

/// Returns whether `at` is on a single-edged blade `long` with its spine on the shaft, or on the
/// guard across the shaft behind it.
fn blade(at: Spot, long: f32, half: f32) -> bool {
    let (t, v) = (at.back, at.side);
    let edge = (0.0..=long).contains(&t) && v >= -half && v <= 0.11 * (t / 0.14).min(1.0);
    edge || ((long..=long + 3.0 * half).contains(&t) && v.abs() <= 0.1)
}

/// Returns whether `at` is on an axe head: the eye round the shaft and the blade that flares out
/// toward its edge.
fn axe(at: Spot) -> bool {
    let (t, v) = (at.back, at.side);
    let eye = (0.10..=0.24).contains(&t) && v.abs() <= 0.07;
    let flare = (0.07..=0.30).contains(&v) && (t - 0.17).abs() <= 0.07 + 0.05 * (v - 0.07) / 0.23;
    eye || flare
}

/// Returns whether `at` is on the fins at the butt of a throwing shaft `length` long.
fn fins(at: Spot, half: f32, length: f32) -> bool {
    let from = length - at.back;
    (0.0..=0.14).contains(&from) && at.side.abs() <= half + 0.06 * (1.0 - from / 0.14)
}

/// Returns whether `at` is on the head of `figure`, for a shaft `half` wide on each side and
/// `length` long.
fn head(figure: Figure, at: Spot, half: f32, length: f32) -> bool {
    let (t, v) = (at.back, at.side);
    match figure {
        Figure::Spear => leaf(at, 0.30, 0.085),
        Figure::Javelin => leaf(at, 0.18, 0.06) || fins(at, half, length),
        Figure::Knife => blade(at, 0.42, half),
        Figure::Blade => blade(at, 0.55, half),
        Figure::Axe => axe(at),
        Figure::Hammer => (0.04..=0.22).contains(&t) && v.abs() <= 0.22,
        Figure::Club => (0.0..=0.5).contains(&t) && v.abs() <= half + 0.09 * (1.0 - t / 0.5),
    }
}

/// Returns the opacity of white at `at` on a tool of `look` whose shaft is `half` wide on each side,
/// or nothing off the tool: the head in the tint of its material, the shaft in that of its own.
fn ink(look: &Look, figure: Figure, at: Spot, half: f32) -> Option<u8> {
    let full = (BUTT.0 - TIP.0).hypot(BUTT.1 - TIP.1);
    let length = full * SPAN[usize::from(look.length.min(3))];
    if head(figure, at, half, length) {
        return Some(look.head.alpha());
    }
    let shaft = (0.0..=length).contains(&at.back) && at.side.abs() <= half;
    shaft.then(|| look.shaft.alpha())
}

/// Returns the picture of `look` drawn `pixels` wide and high, as rows of white pixels from the top
/// with their opacity, or a clear picture when the look has no figure to draw.
pub fn raster(look: &Look, pixels: u32) -> Vec<u8> {
    let side = pixels.max(1) as usize;
    let mut out = vec![0; side * side * BYTES];
    let Some(figure) = look.figure else {
        return out;
    };
    let unit = 1.0 / side as f32;
    let half = (side as f32 / 12.0).round().max(1.0) * unit * 0.5;
    for (at, pixel) in out.as_chunks_mut::<BYTES>().0.iter_mut().enumerate() {
        let x = ((at % side) as f32 + 0.5) * unit - TIP.0;
        let y = ((at / side) as f32 + 0.5) * unit - TIP.1;
        let spot = Spot {
            back: (y - x) * FRAC_1_SQRT_2,
            side: (x + y) * FRAC_1_SQRT_2,
        };
        if let Some(alpha) = ink(look, figure, spot, half) {
            pixel.copy_from_slice(&[255, 255, 255, alpha]);
        }
    }
    out
}

/// Lays the HUD shadow under `out`, a picture `side` pixels wide: every clear pixel `reach` pixels
/// down and to the right of an inked one takes the shadow colour. Nothing changes at a reach of 0.
pub fn shade(out: &mut [u8], side: usize, reach: usize) {
    if reach == 0 {
        return;
    }
    let dark = theme::HUD_SHADOW.to_srgba().to_u8_array();
    let inked = |x: usize, y: usize, out: &[u8]| out[(y * side + x) * BYTES + 3] > 0;
    for at in (0..side * side).rev() {
        let (x, y) = (at % side, at / side);
        let under = x >= reach && y >= reach && inked(x - reach, y - reach, out);
        if under && !inked(x, y, out) {
            out[at * BYTES..(at + 1) * BYTES].copy_from_slice(&dark);
        }
    }
}
