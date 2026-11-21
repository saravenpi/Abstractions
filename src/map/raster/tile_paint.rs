//! Painting one tile from the ground, and finding which of its pixels the explored ground
//! reaches.

use std::collections::HashSet;

use abstractions_net::leaves::Leaves;

use super::BYTES;
use super::paint::colour;
use super::tile::{SIDE, TileId};
use crate::map::explored::grid::{centre as leaf_centre, depth_for, leaf_of};
use crate::world::Globe;

/// Returns the pixels of tile `id` that show explored ground of a body of `radius` metres: the
/// pixels that hold the centre of a leaf of `leaves`, so that a leaf smaller than a pixel shows
/// on a tile seen from far, and the pixels whose own middle lies in a leaf of `leaves`, so that
/// a leaf wider than a pixel is filled on a tile seen from close.
pub fn lit(leaves: &Leaves, id: TileId, radius: f64) -> HashSet<(usize, usize)> {
    let depth = depth_for(radius);
    let held = leaves.iter().filter_map(|leaf| id.pixel(leaf_centre(leaf)));
    let middle = |(px, py): (usize, usize)| id.direction((px as f64 + 0.5, py as f64 + 0.5));
    let inside = (0..SIDE)
        .flat_map(|py| (0..SIDE).map(move |px| (px, py)))
        .filter(|at| leaves.contains(leaf_of(middle(*at), depth)));
    held.chain(inside).collect()
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
