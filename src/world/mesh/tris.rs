//! Terrain triangles: two flat triangles per cell, coloured by height, slope and biome.

use bevy::prelude::*;

use super::patch::Patch;
use super::skirt;
use crate::world::biome::ground_rgb;
use crate::world::shapes::{FlatMesh, srgb};
use crate::world::surface::classify;
use crate::world::{Biome, SEA_LEVEL};

/// How far below sea level a coarse mesh draws its flat sea, so that the real sea covers it.
const SEA_DROP: f32 = 1.5;

/// sRGB colour of the flat sea of a coarse mesh.
const SEA_RGB: [f32; 3] = [0.09, 0.33, 0.5];

/// How a patch is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Lowest height drawn: lower ground is covered by a flat sea at this height.
    pub floor: f32,
    /// How far the skirts hang below the edges, in metres. Zero means no skirts.
    pub skirt: f32,
}

impl Style {
    /// The style of a detail chunk: the real sea bed and the skirts of the 2 m grid.
    pub const DETAIL: Style = Style {
        floor: f32::NEG_INFINITY,
        skirt: 3.0,
    };

    /// Returns the style of a coarse mesh with cells `cell` metres wide: a flat sea and skirts
    /// deep enough to hide the cracks along a finer neighbour.
    pub fn coarse(cell: f32) -> Style {
        Style {
            floor: SEA_LEVEL - SEA_DROP,
            skirt: 3.0 + cell * 0.5,
        }
    }
}

/// Returns a stable value in `[0.92, 1.08]` for a grid cell and triangle index.
pub fn tone(i: usize, j: usize, half: usize) -> f32 {
    let h = (i as u32).wrapping_mul(73_856_093)
        ^ (j as u32).wrapping_mul(19_349_663)
        ^ (half as u32 * 83_492_791);
    let h = (h ^ (h >> 15)).wrapping_mul(0x2C1B_3C6D);
    0.92 + ((h >> 10) & 0xFF) as f32 / 255.0 * 0.16
}

/// Returns the colour of the triangle over vertices `c`: sea when all of it lies on the floor,
/// else the ground for its mean height, its slope against the local up and `biome`.
fn shade(patch: &Patch, c: [[usize; 2]; 3], floor: f32, ground: (Biome, f32)) -> [f32; 4] {
    let p = c.map(|[i, j]| patch.at(i, j));
    let h = c.map(|[i, j]| patch.height(i, j));
    if h.iter().all(|h| *h <= floor) {
        return srgb(SEA_RGB[0], SEA_RGB[1], SEA_RGB[2]);
    }
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let cos = normal.dot(patch.up(c[0][0], c[0][1]));
    let slope = (1.0 - cos * cos).max(0.0).sqrt() / cos.max(0.01);
    let (biome, t) = ground;
    let (r, g, b) = ground_rgb(classify((h[0] + h[1] + h[2]) / 3.0, slope, biome), biome);
    srgb(r * t, g * t, b * t)
}

/// Adds the triangles and skirts of `patch` to `mesh`, asking `biome_at(i, j, half)` for the
/// biome of each triangle.
pub fn fill(
    mesh: &mut FlatMesh,
    patch: &Patch,
    style: Style,
    biome_at: &dyn Fn(usize, usize, usize) -> Biome,
) {
    let n = patch.cells();
    let mut colors = Vec::with_capacity(n * n);
    for (i, j) in (0..n).flat_map(|i| (0..n).map(move |j| (i, j))) {
        let halves = [
            [[i, j], [i, j + 1], [i + 1, j]],
            [[i + 1, j], [i, j + 1], [i + 1, j + 1]],
        ];
        let mut pair = [[0.0; 4]; 2];
        for (half, c) in halves.into_iter().enumerate() {
            let ground = (biome_at(i, j, half), tone(i, j, half));
            pair[half] = shade(patch, c, style.floor, ground);
            let [a, b, c] = c.map(|[i, j]| patch.at(i, j));
            mesh.tri(a, b, c, pair[half]);
        }
        colors.push(pair);
    }
    if style.skirt > 0.0 {
        let drop = |i: usize, j: usize| -patch.up(i, j) * style.skirt;
        let color = |i: usize, j: usize, half: usize| colors[i * n + j][half];
        skirt::add(mesh, n, &|i, j| patch.at(i, j), &color, &drop);
    }
}
