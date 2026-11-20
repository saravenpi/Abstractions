//! Painting a patch: one colour per texel, taken from the same palette as the picture of the
//! full map, and the structures of the planet stamped over it. Plain data, made to run on a
//! worker thread.
//!
//! The budget is one patch at a time and nothing on the main thread but the copy of the
//! finished image. A patch is `TEXELS` squared colours, 9,216 of them, each from a handful of
//! ground samples. Listing the structures of the nearest range adds about 70 milliseconds the
//! first time a worker sees its cells.

use std::sync::Arc;

use bevy::math::{IVec2, Vec2};

use super::patch::{Patch, TEXELS};
use crate::map::raster::colour;
use crate::world::globe::Face;
use crate::world::structures::{CELL, Kind, structure_in_cell};
use crate::world::{Globe, Terrain};

/// Bytes of one texel: red, green, blue and opacity.
pub const BYTES: usize = 4;

/// Largest range in metres for which the structures are listed: the nearest step. The cells
/// to search grow with the square of the range, and at 800 m a hundred stamps would hide the
/// ground for a third of a second of search.
pub const SITES_RANGE: f32 = 250.0;

/// How much wider than the patch, in chart units per metre, the cells of a face are searched:
/// a chart unit is a metre only in the middle of its face.
const SLACK: f32 = 1.5;

/// Returns the image of `patch` on `globe`, row by row from the north: the colour of the
/// planet at each texel, in the palette of the full map with its relief shading, fully opaque.
pub fn paint(globe: &Globe, patch: &Patch) -> Vec<u8> {
    let texels = (0..TEXELS).flat_map(|y| (0..TEXELS).map(move |x| (x, y)));
    texels
        .flat_map(|(x, y)| colour(globe, patch.dir(x, y)))
        .collect()
}

/// Stamps a structure `at` metres east and north of the middle of `patch`: a white dot in a
/// black box, which reads on any ground. A village is a larger one.
pub fn stamp(pixels: &mut [u8], patch: &Patch, at: Vec2, village: bool) {
    let texel = patch.half() * 2.0 / TEXELS as f32;
    let cell = ((Vec2::new(at.x, -at.y) + patch.half()) / texel)
        .floor()
        .as_ivec2();
    let reach = 1 + i32::from(village);
    for (dx, dy) in (-reach..=reach).flat_map(|dy| (-reach..=reach).map(move |dx| (dx, dy))) {
        let (x, y) = (cell.x + dx, cell.y + dy);
        if x.min(y) < 0 || x.max(y) >= TEXELS as i32 {
            continue;
        }
        let grey = u8::from(dx.abs().max(dy.abs()) < reach) * 255;
        let first = (y as usize * TEXELS + x as usize) * BYTES;
        pixels[first..first + 3].fill(grey);
    }
}

/// Returns the structures of the chart `face` that stand on `patch`: where each one is, in
/// metres east and north of the middle, and whether it is a village.
fn on_face(face: &Terrain, patch: &Patch) -> Vec<(Vec2, bool)> {
    let middle = face.chart().locate(patch.chart.origin);
    let (pad, (low, high)) = (patch.half() * SLACK, face.chart().span());
    let lo = ((middle - pad).max(low) / CELL).floor().as_ivec2();
    let hi = ((middle + pad).min(high) / CELL).floor().as_ivec2();
    let cells = (lo.x..=hi.x).flat_map(|x| (lo.y..=hi.y).map(move |y| IVec2::new(x, y)));
    let sites = cells.filter_map(|cell| structure_in_cell(face, cell));
    let placed = sites.map(|site| {
        let pos = face.embed(site.origin.x, site.floor, site.origin.y);
        (patch.locate(pos.normalize()), site.kind == Kind::Village)
    });
    placed
        .filter(|(at, _)| at.abs().max_element() < patch.half())
        .collect()
}

/// Returns every structure that stands on `patch`, on each face of the cube of `globe` the
/// patch may reach. It searches cells, which takes milliseconds: not for the main thread.
pub fn sites(globe: &Arc<Globe>, patch: &Patch) -> Vec<(Vec2, bool)> {
    let near = |face: &Face| face.axes()[0].dot(patch.chart.origin) > 0.5;
    let faces = Face::ALL.into_iter().filter(near);
    let rooted = |face| globe.face_chart(face).near(patch.chart.origin);
    let charts = faces.map(|face| Terrain::on(globe.clone(), rooted(face)));
    charts.flat_map(|face| on_face(&face, patch)).collect()
}

/// Returns the finished image of `patch`: the ground, and the structures on it while the
/// range is short enough to list them.
pub fn render(globe: &Arc<Globe>, patch: &Patch) -> Vec<u8> {
    let mut pixels = paint(globe, patch);
    if patch.range <= SITES_RANGE {
        for (at, village) in sites(globe, patch) {
            stamp(&mut pixels, patch, at, village);
        }
    }
    pixels
}
