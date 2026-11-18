//! The deck of a gas giant: a smooth sphere at the sea level radius, coloured in soft latitude bands.
//!
//! Nothing here samples the ground. The bands follow the latitude, and a little seeded noise bends them
//! in the colour only, so a deck of any size costs the same per vertex.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::frame::Frame;
use super::patch::Patch;
use super::skirt;
use super::span::Span;
use crate::world::Globe;
use crate::world::islands::gradient;
use crate::world::shapes::FlatMesh;

/// Cells along one side of every deck node: 230 m cells on the deepest nodes of a 150 km giant, which
/// leaves the flat cells a few centimetres under the sphere.
pub const CELLS: usize = 16;

/// Bands from pole to pole, light and dark together counting as two.
const BANDS: f64 = 9.0;

/// How far the seeded noise moves a band, in radians of its wave.
const WOBBLE: f64 = 0.9;

/// Waves of the noise around the planet: a few large swirls, never the grain of ground.
const SWIRL: f64 = 2.5;

/// Returns the linear RGBA colour of the deck of `globe` in unit direction `dir`: the sea and land tones
/// of its look, blended by a wave of the latitude that the seeded noise bends.
pub fn band(globe: &Globe, dir: DVec3) -> [f32; 4] {
    let look = globe.record().class.look();
    let latitude = dir.y.clamp(-1.0, 1.0).asin();
    let swirl = gradient(
        (globe.seed() ^ globe.seed() >> 32) as u32,
        (dir * SWIRL).to_array(),
    );
    let wave = (latitude * BANDS + swirl * WOBBLE).sin();
    let t = (0.5 + 0.5 * wave) as f32;
    let rgb = look.sea.lerp(look.land, t * t * (3.0 - 2.0 * t));
    [rgb.x, rgb.y, rgb.z, 1.0]
}

/// Adds the deck over `span` to `mesh` in `frame`: two flat triangles per cell at height 0, both in the
/// band colour of the middle of the cell, and skirts `skirt` metres deep when that is above 0.
pub fn fill(mesh: &mut FlatMesh, span: &Span, frame: &Frame, skirt: f32) {
    let n = span.n;
    let flat = vec![vec![0.0; n + 1]; n + 1];
    let patch = Patch::new(span, frame, &flat, f32::NEG_INFINITY);
    let globe = span.terrain.globe();
    let middle = |i: usize, j: usize| (span.dir(i, j) + span.dir(i + 1, j + 1)).normalize();
    let colors: Vec<[f32; 4]> = (0..n * n)
        .map(|k| band(globe, middle(k / n, k % n)))
        .collect();
    for (k, color) in colors.iter().enumerate() {
        let (i, j) = (k / n, k % n);
        let at = |a: usize, b: usize| patch.at(a, b);
        mesh.tri(at(i, j), at(i, j + 1), at(i + 1, j), *color);
        mesh.tri(at(i + 1, j), at(i, j + 1), at(i + 1, j + 1), *color);
    }
    if skirt > 0.0 {
        let drop = |i: usize, j: usize| -patch.up(i, j) * skirt;
        let color = |i: usize, j: usize, _: usize| colors[i * n + j];
        skirt::add(mesh, n, &|i, j| patch.at(i, j), &color, &drop);
    }
}
