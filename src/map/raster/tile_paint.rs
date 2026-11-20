//! Painting one tile from the ground, and finding which of its pixels the explored ground
//! reaches.

use std::collections::HashSet;

use abstractions_net::leaves::Leaves;

use super::BYTES;
use super::paint::colour;
use super::tile::{SIDE, TileId};
use crate::map::explored::grid::centre as leaf_centre;
use crate::world::Globe;

/// Returns the pixels of tile `id` that hold the centre of a leaf of `leaves`.
pub fn lit(leaves: &Leaves, id: TileId) -> HashSet<(usize, usize)> {
    leaves
        .iter()
        .filter_map(|leaf| id.pixel(leaf_centre(leaf)))
        .collect()
}

/// Paints tile `id` of `globe`: the colour of the ground at the middle of each pixel, black
/// where `lit` is given and does not hold the pixel. Returns [`SIDE`] rows of [`SIDE`] pixels
/// of [`BYTES`] bytes.
pub fn paint(globe: &Globe, id: TileId, lit: Option<&HashSet<(usize, usize)>>) -> Vec<u8> {
    let mut data = Vec::with_capacity(SIDE * SIDE * BYTES);
    for py in 0..SIDE {
        for px in 0..SIDE {
            let seen = lit.is_none_or(|lit| lit.contains(&(px, py)));
            let pixel = match seen {
                true => colour(globe, id.direction((px as f64 + 0.5, py as f64 + 0.5))),
                false => [0, 0, 0, u8::MAX],
            };
            data.extend_from_slice(&pixel);
        }
    }
    data
}
