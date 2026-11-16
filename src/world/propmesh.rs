//! The plant vocabulary and the meshes of the solid props: trees, cacti, rocks, wrecks, corals and sea life.

mod land;
mod merge;
mod plant;
mod sea;
mod small;
#[cfg(test)]
mod tests;

use std::ops::{Range, RangeInclusive};

use bevy::prelude::*;

use super::shapes::{FlatMesh, shade};
use super::solids::{cone, fan, frustum, icosa, slab};

pub use merge::{merge_props, merge_with};
pub use plant::Plant;

/// Number of mesh variants built for every solid plant.
pub const VARIANTS: usize = 3;

/// Number of big plants, which come first in [`Plant`]; the ground cover follows.
pub const BIG: usize = Plant::Chest as usize + 1;

/// Number of tree kinds, which come first in [`Plant`].
pub const TREES: usize = Plant::Willow as usize + 1;

/// Index of the first flower in [`Plant`].
pub const FLOWER: usize = Plant::Poppy as usize;

/// Indices of the small plants that are built from solid parts instead of blade fans.
pub const PARTS: Range<usize> = Plant::MushroomRing as usize..Plant::Kelp as usize;

/// Indices of the plants whose colors turn with the variant, which gives each coral three palettes.
const SPINS: RangeInclusive<usize> = Plant::BranchCoral as usize..=Plant::Anemone as usize;

/// One solid of a prop, in sRGB colors.
#[derive(Clone, Copy)]
enum Part {
    /// Closed tube from a point to a point: radii at both ends, color.
    Stem([f32; 3], [f32; 3], [f32; 2], [f32; 3]),
    /// Closed cone: base center, radius and height, color.
    Cone([f32; 3], [f32; 2], [f32; 3]),
    /// Jittered ball: center, radii, color.
    Ball([f32; 3], [f32; 3], [f32; 3]),
    /// Double sided fan of leaves: center, fan shape, color.
    Fronds([f32; 3], [f32; 5], [f32; 3]),
    /// Closed box: center, half sizes, roll around the x axis, color.
    Slab([f32; 3], [f32; 3], f32, [f32; 3]),
}

use Part::*;

impl Part {
    /// Returns the number of triangles the part adds to a mesh.
    fn tris(&self) -> usize {
        match *self {
            Stem(..) => 24,
            Cone(..) => 14,
            Ball(..) => 20,
            Slab(..) => 12,
            Fronds(_, f, _) => f[0] as usize * if f[3] == 0.0 { 2 } else { 6 },
        }
    }
}

/// Returns the parts of solid plant number `kind`, an index into [`Plant`].
fn spec(kind: usize) -> &'static [Part] {
    let land = land::SPECS.len();
    if kind < land {
        land::SPECS[kind]
    } else if kind < BIG {
        sea::SPECS[kind - land]
    } else {
        small::SPECS[kind - PARTS.start]
    }
}

/// Returns how many triangles solid plant `kind` has.
pub fn solid_tris(kind: usize) -> usize {
    spec(kind).iter().map(Part::tris).sum()
}

/// Rotates the color channels by `variant` for corals, and leaves other colors alone.
fn spin(c: [f32; 3], kind: usize, variant: usize) -> [f32; 3] {
    let n = if SPINS.contains(&kind) { variant } else { 0 };
    [c[n % 3], c[(n + 1) % 3], c[(n + 2) % 3]]
}

/// Appends part number `i` of plant `kind`, variant `variant`, to a mesh.
fn add_part(m: &mut FlatMesh, kind: usize, variant: usize, i: usize) {
    let v = variant as f32 - 1.0;
    let s = Vec3::new(1.0 - 0.05 * v, 1.0 + 0.1 * v, 1.0 - 0.05 * v);
    let at = |a: [f32; 3]| Vec3::from(a) * s;
    let paint = |c: [f32; 3]| shade(spin(c, kind, variant), 1.0 + 0.05 * v);
    match spec(kind)[i] {
        Stem(a, b, r, c) => frustum(m, at(a), at(b), Vec2::from(r) * s.x, paint(c)),
        Cone(a, r, c) => cone(m, at(a), Vec2::new(r[0] * s.x, r[1] * s.y), 7, paint(c)),
        Ball(a, r, c) => {
            let shape = (0.22, (variant * 31 + i) as u32);
            icosa(m, at(a), Vec3::from(r) * s, shape, paint(c));
        }
        Fronds(a, f, c) => fan(m, at(a), f, variant as f32 * 0.8, paint(c)),
        Slab(a, h, roll, c) => slab(m, at(a), Vec3::from(h) * s, roll, paint(c)),
    }
}

/// Builds the mesh of solid plant number `kind` (an index into [`Plant`]), variant `variant`, base at the origin.
pub fn big_mesh(kind: usize, variant: usize) -> Mesh {
    let mut m = FlatMesh::default();
    for i in 0..spec(kind).len() {
        add_part(&mut m, kind, variant, i);
    }
    m.into_mesh()
}
