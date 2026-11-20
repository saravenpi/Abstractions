//! Terrain triangles: two flat triangles per cell, coloured by height, slope and biome.
//!
//! The colour of a triangle comes from the one rule of the lands, read at a fixed scale at the cell centre
//! and never from the facet of this mesh, so a coarse level paints a place as a fine one does.

use bevy::prelude::*;

use super::patch::Patch;
use super::read::{TURF_BAND, TURF_LEVELS, facet, level, read, rock_share};
use super::skirt;
use super::style::{Style, tone};
use crate::world::lands::Reading;
use crate::world::shapes::{FlatMesh, srgb};
use crate::world::{Biome, Surface};

/// What the ground of one cell is: its biome, the biomes dithered for its two triangles, the surface a
/// channel bed puts on it above the sea, and its height and slope at the fixed scale of the colours, where
/// the chart has a globe to read them from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    /// The biome of the cell, then the biomes dithered for its two triangles.
    pub biomes: [Biome; 3],
    /// The surface a channel bed puts on the cell, where one runs.
    pub bed: Option<Surface>,
    /// Its height and slope read at the cell centre, or nothing on the legacy plane.
    pub reading: Option<Reading>,
}

/// Returns the sRGB colour the home planet gives ground at `height` and `slope` in the biome `base`,
/// tinted by the dithered biome `near`.
#[cfg(test)]
pub fn ground(height: f32, slope: f32, base: Biome, near: Biome) -> (f32, f32, f32) {
    let home = crate::world::lands::Lands::of(&crate::universe::PlanetRecord::TEMPERATE);
    home.colour(Reading { height, slope }, [base, near], None).0
}

/// Slope far over the rock threshold, to read the rock a place would be.
const STEEP: f32 = 100.0;

/// What one triangle is painted from: its cell, which of the two triangles of the cell it is, and the gain of its tone.
type Ground = (Cell, usize, f32);

/// Returns the wrapped chart position of a vertex and how thick the turf grows there.
pub type Turf<'a> = &'a dyn Fn(usize, usize) -> [f32; 3];

/// Returns the colour of the triangle over vertices `c` and its surface: sea when all of it lies on
/// the floor, else the ground at the reading of its cell, in the biome of the cell and the biome dithered
/// for the triangle, with grass moved to the mean colour of its carpet.
fn shade(
    patch: &Patch,
    c: [[usize; 2]; 3],
    (style, turf): (&Style, Option<Turf>),
    (cell, half, gain): Ground,
) -> ([f32; 4], Surface) {
    let h = c.map(|[i, j]| patch.height(i, j));
    if h.iter().all(|h| *h <= style.floor) {
        let [r, g, b] = style.lands.sea_rgb();
        return (srgb(r, g, b), Surface::Water);
    }
    let biomes = [cell.biomes[0], cell.biomes[1 + half]];
    let reading = read(h.iter().sum::<f32>() / 3.0, cell.reading, || {
        facet(patch, c)
    });
    let classed = |slope: f32| {
        style
            .lands
            .colour(Reading { slope, ..reading }, biomes, cell.bed)
    };
    let (open, surface) = match cell.reading {
        Some(_) => classed(0.0),
        None => classed(reading.slope),
    };
    let open = carpeted(style, turf, c, (open, surface));
    let (rgb, surface) = match cell.reading {
        Some(r) => {
            let (rock, kind) = classed(STEEP);
            let share = rock_share(r.slope);
            let mixed = Vec3::from(open).lerp(Vec3::from(rock), share);
            let rgb: (f32, f32, f32) = mixed.into();
            (rgb, if share > 0.5 { kind } else { surface })
        }
        None => (open, surface),
    };
    (srgb(rgb.0 * gain, rgb.1 * gain, rgb.2 * gain), surface)
}

/// Returns the colour `ground` of a triangle over vertices `c` with grass moved to the mean colour of its carpet.
fn carpeted(
    style: &Style,
    turf: Option<Turf>,
    c: [[usize; 2]; 3],
    ground: ((f32, f32, f32), Surface),
) -> (f32, f32, f32) {
    match (style.verdure, ground.1) {
        (Some(verdure), Surface::Grass) => {
            let thick = turf.map_or(verdure.cover, |turf| {
                level(&c.map(|[i, j]| turf(i, j))) / TURF_LEVELS
            });
            verdure.carpeted(ground.0, thick)
        }
        _ => ground.0,
    }
}

/// Returns the turf coordinates of the three vertices `c` of a triangle of `surface`: their wrapped
/// chart positions from `turf`, lifted by the band of the quarters of turf the triangle holds on open grass.
fn turfed(turf: Turf, c: [[usize; 2]; 3], surface: Surface) -> [[f32; 2]; 3] {
    let v = c.map(|[i, j]| turf(i, j));
    let lift = match surface {
        Surface::Grass => level(&v) * TURF_BAND,
        _ => 0.0,
    };
    v.map(|[x, z, _]| [x, z + lift])
}

/// Adds the two triangles of cell `(i, j)` of `patch` to `mesh`, and returns their colours.
fn paint_cell(
    mesh: &mut FlatMesh,
    patch: &Patch,
    (i, j): (usize, usize),
    (style, turf): (&Style, Option<Turf>),
    ground: Cell,
) -> [[f32; 4]; 2] {
    let halves = [
        [[i, j], [i, j + 1], [i + 1, j]],
        [[i + 1, j], [i, j + 1], [i + 1, j + 1]],
    ];
    let mut pair = [[0.0; 4]; 2];
    for (half, c) in halves.into_iter().enumerate() {
        let toned = if style.toned { tone(i, j, half) } else { 1.0 };
        let (colour, surface) = shade(patch, c, (style, turf), (ground, half, toned));
        pair[half] = colour;
        if let Some(turf) = turf {
            mesh.turf(turfed(turf, c, surface));
        }
        let [a, b, c] = c.map(|[i, j]| patch.at(i, j));
        mesh.tri(a, b, c, colour);
    }
    pair
}

/// Adds the triangles and skirts of `patch` to `mesh`, asking `cell_at(i, j)` for the biome of each cell,
/// the biome dithered for each of its two triangles, its channel bed and its reading, and `turf(i, j)`,
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
        let ground = cell_at(i, j);
        colors.push(paint_cell(mesh, patch, (i, j), (&style, turf), ground));
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
