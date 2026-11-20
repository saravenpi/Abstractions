//! Painting a patch: one colour per texel, taken from the same palette as the picture of the
//! full map. Plain data, made to run on a worker thread. The minimap shows the ground only: no
//! structure is stamped on it.
//!
//! The budget is one patch at a time and nothing on the main thread but the copy of the
//! finished image. A patch is `TEXELS` squared colours, 9,216 of them, each from a handful of
//! ground samples.

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
