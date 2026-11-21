//! Painting one tile from the ground, and finding which of its pixels the explored ground
//! reaches.

use std::collections::HashSet;

use std::f64::consts::TAU;

use abstractions_net::leaves::Leaves;
use bevy::math::{DVec3, Vec2};

use super::paint::{colour, shaded};
use super::tile::{SIDE, TileId};
use super::{BYTES, UNSEEN};
use crate::map::explored::grid::{centre as leaf_centre, depth_for, leaf_of};
use crate::world::Globe;
use crate::world::globe::Grain;

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

/// Pixels of the grid of heights a close tile is read from along one side: the tile and a ring
/// of one pixel around it, for the slope at its edges.
const RING: usize = SIDE + 2;

/// Returns the metres between the middles of two pixels of a tile of `level` along the equator
/// of a body `radius` metres in radius.
fn spacing(level: u8, radius: f64) -> f64 {
    TAU * radius / (f64::from(TileId::cols(level)) * SIDE as f64)
}

/// Returns the heights of the ground of `globe` at the middle of every pixel of tile `id` and of
/// the ring around it, as a reader of `grain` sees them, row by row from the north-west.
fn heights(globe: &Globe, id: TileId, grain: Grain) -> Vec<f32> {
    let at = |i: usize| {
        let (px, py) = ((i % RING) as f64 - 0.5, (i / RING) as f64 - 0.5);
        globe.height_seen(id.direction((px, py)), grain)
    };
    (0..RING * RING).map(at).collect()
}

/// Returns the rise towards the east and the north at pixel `(px, py)` of a tile whose heights
/// are `heights`, its pixels `metres` apart on the equator, in unit direction `dir`.
fn rise(heights: &[f32], (px, py): (usize, usize), metres: f64, dir: DVec3) -> Vec2 {
    let h = |x: usize, y: usize| heights[y * RING + x];
    let (x, y) = (px + 1, py + 1);
    let across = (metres * (1.0 - dir.y * dir.y).sqrt().max(1e-3)) as f32;
    let east = (h(x + 1, y) - h(x - 1, y)) / (2.0 * across);
    let north = (h(x, y - 1) - h(x, y + 1)) / (2.0 * metres as f32);
    Vec2::new(east, north)
}

/// Paints tile `id` of `globe`: the colour of the ground at the middle of each pixel, clear
/// where `lit` is given and does not hold the pixel. A tile whose pixels are close enough to
/// show more than the bare shape of the land reads the ground itself at their grain, with its
/// slope and the light on it. Returns [`SIDE`] rows of [`SIDE`] pixels of [`BYTES`] bytes.
pub fn paint(globe: &Globe, id: TileId, lit: Option<&HashSet<(usize, usize)>>) -> Vec<u8> {
    let metres = spacing(id.level, globe.radius());
    let grain = Grain::of(metres as f32);
    let close = grain != Grain::Far && lit.is_none_or(|lit| !lit.is_empty());
    let ground = close.then(|| heights(globe, id, grain));
    let mut data = Vec::with_capacity(SIDE * SIDE * BYTES);
    for py in 0..SIDE {
        for px in 0..SIDE {
            let seen = lit.is_none_or(|lit| lit.contains(&(px, py)));
            let dir = id.direction((px as f64 + 0.5, py as f64 + 0.5));
            let pixel = match (seen, &ground) {
                (false, _) => UNSEEN,
                (true, None) => colour(globe, dir),
                (true, Some(h)) => {
                    let slope = rise(h, (px, py), metres, dir);
                    shaded(globe, (dir, grain), h[(py + 1) * RING + px + 1], slope)
                }
            };
            data.extend_from_slice(&pixel);
        }
    }
    data
}
