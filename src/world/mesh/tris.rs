//! Terrain triangles: two flat triangles per cell, coloured by height, slope and biome.

use bevy::prelude::*;

use super::patch::Patch;
use super::skirt;
use crate::world::lands::Lands;
use crate::world::shapes::{FlatMesh, srgb};
use crate::world::{Biome, SEA_LEVEL, Surface};

/// What the ground of one cell is: its biome, the biomes dithered for its two triangles, and the surface a
/// channel bed puts on it above the sea.
pub type Cell = ([Biome; 3], Option<Surface>);

/// Quarters of turf a triangle can hold, and the lift in metres of the second uv `y` per quarter: the
/// `LEVELS` and `BAND` of `turf.wesl`.
const TURF_LEVELS: f32 = 4.0;

/// Lift of the second uv `y` per quarter of turf.
const TURF_BAND: f32 = 512.0;

/// How far below sea level a coarse mesh draws its flat sea, so that the real sea covers it.
const SEA_DROP: f32 = 1.5;

/// How a patch is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// Lowest height drawn: lower ground is covered by a flat sea at this height.
    pub floor: f32,
    /// How far the skirts hang below the edges, in metres. Zero means no skirts.
    pub skirt: f32,
    /// The ground rules of the planet, which give the surfaces and their colours.
    pub lands: Lands,
}

impl Style {
    /// Returns the style of a detail chunk on `lands`: the real sea bed and the skirts of the 2 m grid.
    pub fn detail(lands: Lands) -> Style {
        Style {
            floor: f32::NEG_INFINITY,
            skirt: 3.0,
            lands,
        }
    }

    /// Returns the style of a coarse mesh on `lands` with cells `cell` metres wide: a flat sea where the
    /// planet has one, and skirts deep enough to hide the cracks along a finer neighbour.
    pub fn coarse(cell: f32, lands: Lands) -> Style {
        let floor = match lands.has_sea() {
            true => SEA_LEVEL - SEA_DROP,
            false => f32::NEG_INFINITY,
        };
        Style {
            floor,
            skirt: 3.0 + cell * 0.5,
            lands,
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

/// Returns the sRGB colour of ground of `lands` at `height` and `slope` in the biome `base` of its cell,
/// tinted by the biome `near` dithered across a border when that biome covers it the same way. A channel
/// `bed` above the sea takes the place of the surface.
///
/// The surface always follows `base`: a dithered biome that would turn sand into grass or grass
/// into rock would leave single triangles of the wrong surface on a border.
fn paint(
    lands: Lands,
    height: f32,
    slope: f32,
    [base, near]: [Biome; 2],
    bed: Option<Surface>,
) -> ((f32, f32, f32), Surface) {
    let surface = match bed {
        Some(bed) if height > SEA_LEVEL => bed,
        _ => lands.surface(height, slope, base),
    };
    let tint = match lands.surface(height, slope, near) == surface {
        true => near,
        false => base,
    };
    (lands.rgb(surface, tint, height), surface)
}

/// Returns the sRGB colour the home planet gives ground at `height` and `slope` in the biome `base`,
/// tinted by the dithered biome `near`.
#[cfg(test)]
pub fn ground(height: f32, slope: f32, base: Biome, near: Biome) -> (f32, f32, f32) {
    let home = Lands::of(&crate::universe::PlanetRecord::TEMPERATE);
    paint(home, height, slope, [base, near], None).0
}

/// What one triangle is painted from: its biome and the one dithered for it, a channel bed, and its tone.
type Ground = ([Biome; 2], Option<Surface>, f32);

/// Returns the colour of the triangle over vertices `c` and its surface: sea when all of it lies on
/// the floor, else the ground for its mean height and its slope against the local up, in the biome
/// of its cell and the biome dithered for it.
fn shade(patch: &Patch, c: [[usize; 2]; 3], style: &Style, ground: Ground) -> ([f32; 4], Surface) {
    let p = c.map(|[i, j]| patch.at(i, j));
    let h = c.map(|[i, j]| patch.height(i, j));
    if h.iter().all(|h| *h <= style.floor) {
        let [r, g, b] = style.lands.sea_rgb();
        return (srgb(r, g, b), Surface::Water);
    }
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
    let cos = normal.dot(patch.up(c[0][0], c[0][1]));
    let slope = (1.0 - cos * cos).max(0.0).sqrt() / cos.max(0.01);
    let (biomes, bed, t) = ground;
    let ((r, g, b), surface) = paint(style.lands, (h[0] + h[1] + h[2]) / 3.0, slope, biomes, bed);
    (srgb(r * t, g * t, b * t), surface)
}

/// Returns the wrapped chart position of a vertex and how thick the turf grows there.
pub type Turf<'a> = &'a dyn Fn(usize, usize) -> [f32; 3];

/// Returns the turf coordinates of the three vertices `c` of a triangle of `surface`: their wrapped
/// chart positions from `turf`, lifted by the band of the mean thickness of the turf on open grass.
fn turfed(turf: Turf, c: [[usize; 2]; 3], surface: Surface) -> [[f32; 2]; 3] {
    let v = c.map(|[i, j]| turf(i, j));
    let thick = match surface {
        Surface::Grass => (v[0][2] + v[1][2] + v[2][2]) / 3.0,
        _ => 0.0,
    };
    let lift = (thick.clamp(0.0, 1.0) * TURF_LEVELS).round() * TURF_BAND;
    v.map(|[x, z, _]| [x, z + lift])
}

/// Adds the triangles and skirts of `patch` to `mesh`, asking `cell_at(i, j)` for the biome of
/// each cell, the biome dithered for each of its two triangles and its channel bed, and `turf(i, j)`,
/// when given, for the wrapped chart position of each vertex and how thick the turf grows there.
pub fn fill(
    mesh: &mut FlatMesh,
    patch: &Patch,
    (style, turf): (Style, Option<Turf>),
    cell_at: &dyn Fn(usize, usize) -> Cell,
) {
    let n = patch.cells();
    let mut colors = Vec::with_capacity(n * n);
    for (i, j) in (0..n).flat_map(|i| (0..n).map(move |j| (i, j))) {
        let halves = [
            [[i, j], [i, j + 1], [i + 1, j]],
            [[i + 1, j], [i, j + 1], [i + 1, j + 1]],
        ];
        let mut pair = [[0.0; 4]; 2];
        let ([base, first, second], bed) = cell_at(i, j);
        for (half, c) in halves.into_iter().enumerate() {
            let ground = ([base, [first, second][half]], bed, tone(i, j, half));
            let (colour, surface) = shade(patch, c, &style, ground);
            pair[half] = colour;
            if let Some(turf) = turf {
                mesh.turf(turfed(turf, c, surface));
            }
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

#[cfg(test)]
#[path = "tests_tris.rs"]
mod tests;
