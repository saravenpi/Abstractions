//! Painting a patch: one colour per texel, taken from the same palette as the picture of the
//! full map, from the planet's own ground functions and never from the terrain the world has
//! loaded. Plain data, made to run on a worker thread. The minimap shows the ground only.
//!
//! The budget is one patch at a time and nothing on the main thread but the copy of the
//! finished image. A patch is `TEXELS` squared colours, 9,216 of them, each from a handful of
//! ground samples. A patch moved by whole texels keeps the texels of the one before and samples
//! only its new edge.

use super::patch::{Patch, TEXELS};
use crate::map::raster::colour;
use crate::world::Globe;

/// Bytes of one texel: red, green, blue and opacity.
pub const BYTES: usize = 4;

/// Returns the image of `patch` on `globe`, row by row from the north: the colour of the
/// planet at each texel, in the palette of the full map with its relief shading, fully opaque.
pub fn paint(globe: &Globe, patch: &Patch) -> Vec<u8> {
    let texels = (0..TEXELS).flat_map(|y| (0..TEXELS).map(move |x| (x, y)));
    texels
        .flat_map(|(x, y)| colour(globe, patch.dir(x, y)))
        .collect()
}

/// Returns the image of `patch` on `globe` as [`paint`] does, taking the texels it shares with
/// the image `old` of an earlier patch from that image when `patch` is that patch moved by whole
/// texels, so only the new edge is sampled. Anything else is painted whole.
pub fn repaint(globe: &Globe, patch: &Patch, old: Option<(&Patch, &[u8])>) -> Vec<u8> {
    let full = TEXELS * TEXELS * BYTES;
    let reused = old.filter(|(_, pixels)| pixels.len() == full);
    let Some((shift, pixels)) =
        reused.and_then(|(was, pixels)| Some((patch.texels_in(was)?, pixels)))
    else {
        return paint(globe, patch);
    };
    let side = TEXELS as i32;
    let mut data = Vec::with_capacity(full);
    for y in 0..TEXELS {
        for x in 0..TEXELS {
            let (ox, oy) = (x as i32 + shift.x, y as i32 + shift.y);
            match (0..side).contains(&ox) && (0..side).contains(&oy) {
                true => {
                    let at = (oy as usize * TEXELS + ox as usize) * BYTES;
                    data.extend_from_slice(&pixels[at..at + BYTES]);
                }
                false => data.extend_from_slice(&colour(globe, patch.dir(x, y))),
            }
        }
    }
    data
}
