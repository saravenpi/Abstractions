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

/// Returns whether `at` is on a knife or double-edged sword proportional to `length`.
fn blade(at: Spot, sword: bool, half: f32, length: f32) -> bool {
    let (t, v) = (at.back, at.side);
    if !sword {
        let long = 0.52 * length;
        let belly = -half + (half + 0.115) * (t / (0.28 * long).max(0.05)).min(1.0);
        let edge = (0.0..=long).contains(&t) && v >= -half && v <= belly;
        return edge || ((long..=long + 0.035).contains(&t) && v.abs() <= half * 1.9);
    }
    let long = 0.56 * length;
    let edge =
        (0.0..=long).contains(&t) && v.abs() <= 0.08 * (t / (0.25 * long).max(0.05)).min(1.0);
    let guard = (long..=long + 0.055).contains(&t) && v.abs() <= 0.165;
    let pommel = (length - 0.055..=length).contains(&t) && v.abs() <= 0.055;
    edge || guard || pommel
}

/// Returns whether `at` is on an axe head: a compact poll opposite the shaft and a bearded blade
/// flaring toward a curved cutting edge.
fn axe(at: Spot) -> bool {
    let (t, v) = (at.back, at.side);
    let poll = (0.09..=0.20).contains(&t) && (-0.09..=0.04).contains(&v);
    let s = ((v - 0.03) / 0.29).clamp(0.0, 1.0);
    let top = 0.09 - 0.045 * s;
    let bot = 0.20 + 0.14 * s;
    let edge = 0.32 - 0.06 * ((t - 0.19) / 0.15).powi(2);
    poll || ((0.03..=edge).contains(&v) && (top..=bot).contains(&t))
}

/// Returns whether `at` is on a hammer head: a narrow socket, wide striking face and tapered peen.
fn hammer(at: Spot) -> bool {
    let (t, v) = (at.back, at.side);
    let face = (0.03..=0.21).contains(&t) && (0.09..=0.27).contains(&v);
    let neck = (0.07..=0.17).contains(&t) && (-0.07..=0.10).contains(&v);
    let s = ((-0.07 - v) / 0.18).clamp(0.0, 1.0);
    let peen = (-0.25..=-0.06).contains(&v) && (t - 0.12).abs() <= 0.07 - 0.035 * s;
    face || neck || peen
}

/// Returns whether `at` is on the head of `figure`, for a shaft `half` wide on each side and
/// `length` long.
fn head(figure: Figure, at: Spot, half: f32, length: f32) -> bool {
    let (t, v) = (at.back, at.side);
    match figure {
        Figure::Spear => leaf(at, 0.30, 0.085),
        Figure::Javelin => {
            let tail = length - t;
            let fins = (0.0..=0.14).contains(&tail) && v.abs() <= half + 0.06 * (1.0 - tail / 0.14);
            leaf(at, 0.18, 0.06) || fins
        }
        Figure::Knife => blade(at, false, half, length),
        Figure::Blade => blade(at, true, half, length),
        Figure::Axe => axe(at),
        Figure::Hammer => hammer(at),
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
    let start = match figure {
        Figure::Blade => 0.56 * length,
        Figure::Knife => 0.52 * length,
        Figure::Spear => 0.15,
        Figure::Javelin => 0.09,
        Figure::Axe | Figure::Hammer => 0.06,
        Figure::Club => 0.0,
    };
    let shaft = (start..=length).contains(&at.back) && at.side.abs() <= half;
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
