//! A cache of the drawn ground: the heights of the terrain mesh at its grid points, kept from frame
//! to frame, so animals that step on the same ground every frame sample the terrain once.
//!
//! Reading the drawn ground is the cost of a walking animal: every foot and every slope probe asks
//! for it, and each answer reads three terrain heights. The heights at grid points do not change
//! while the terrain view stays, so a small table of them answers most reads.

use std::cell::Cell;

use crate::world::Terrain;

/// Side in metres of the triangles of the finest terrain mesh, `crate::world::mesh::GRID_STEP`.
/// The cache test checks that both read the same ground.
const STEP: f32 = 2.0;

/// Grid points the cache holds.
const SLOTS: usize = 4096;

/// An empty slot.
const EMPTY: (i32, i32, f32) = (i32::MIN, i32::MIN, 0.0);

/// The cached heights, by grid point, direct mapped.
pub struct Heights {
    /// Grid point and height of each slot.
    slots: Box<[Cell<(i32, i32, f32)>]>,
}

impl Default for Heights {
    fn default() -> Self {
        Self {
            slots: (0..SLOTS).map(|_| Cell::new(EMPTY)).collect(),
        }
    }
}

impl Heights {
    /// Forgets every height, as when the terrain view changed.
    pub fn clear(&self) {
        for slot in self.slots.iter() {
            slot.set(EMPTY);
        }
    }

    /// Returns the height of `terrain` at grid point `(i, j)`, read once.
    fn point(&self, terrain: &Terrain, i: i32, j: i32) -> f32 {
        let hash = (i.wrapping_mul(73_856_093) ^ j.wrapping_mul(19_349_663)) as u32 as usize;
        let slot = &self.slots[hash % SLOTS];
        let (si, sj, h) = slot.get();
        if (si, sj) == (i, j) {
            return h;
        }
        let h = terrain.height(i as f32 * STEP, j as f32 * STEP);
        slot.set((i, j, h));
        h
    }

    /// Returns the height the finest terrain mesh draws at chart position `(x, z)` of `terrain`:
    /// what `Terrain::mesh_height` returns, from cached grid points.
    pub fn mesh(&self, terrain: &Terrain, x: f32, z: f32) -> f32 {
        let (i, j) = ((x / STEP).floor(), (z / STEP).floor());
        let (u, v) = (x / STEP - i, z / STEP - j);
        let (i, j) = (i as i32, j as i32);
        let h = |a: i32, b: i32| self.point(terrain, i + a, j + b);
        if u + v <= 1.0 {
            h(0, 0) + u * (h(1, 0) - h(0, 0)) + v * (h(0, 1) - h(0, 0))
        } else {
            h(1, 1) + (1.0 - u) * (h(0, 1) - h(1, 1)) + (1.0 - v) * (h(1, 0) - h(1, 1))
        }
    }
}
