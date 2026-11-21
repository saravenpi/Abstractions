//! A cache of the drawn ground: the heights of the terrain mesh at its grid points, kept from frame
//! to frame, so animals that step on the same ground every frame sample the terrain once.
//!
//! Reading the drawn ground is the cost of a walking animal: every foot and every slope probe asks
//! for it, and each answer reads three terrain heights. The heights at grid points do not change
//! while the terrain view stays, so a small table of them answers most reads. Which grid a place is
//! drawn on changes as chunks stream, so that table lives one frame.

use std::cell::Cell;

use bevy::math::{IVec2, Vec2};

use crate::world::{CHUNK_SIZE, Chunks, Drawn, Terrain};

/// Side in metres of the triangles of the finest terrain mesh, `crate::world::mesh::GRID_STEP`.
/// The cache test checks that both read the same ground.
const STEP: f32 = 2.0;

/// Grid points the cache holds.
const SLOTS: usize = 4096;

/// Chunks whose drawn grid the cache holds.
const GRIDS: usize = 64;

/// An empty slot.
const EMPTY: (i32, i32, f32) = (i32::MIN, i32::MIN, 0.0);

/// An empty grid slot.
const NO_GRID: (IVec2, Option<Drawn>) = (IVec2::MIN, None);

/// A chunk and the grid drawn over it, if any.
type Slot = Cell<(IVec2, Option<Drawn>)>;

/// The cached heights, by grid point, direct mapped, and the drawn grids, by chunk.
pub struct Heights {
    /// Grid point and height of each slot.
    slots: Box<[Cell<(i32, i32, f32)>]>,
    /// Chunk and the grid drawn over it of each slot.
    grids: Box<[Slot]>,
}

impl Default for Heights {
    fn default() -> Self {
        Self {
            slots: (0..SLOTS).map(|_| Cell::new(EMPTY)).collect(),
            grids: (0..GRIDS).map(|_| Cell::new(NO_GRID)).collect(),
        }
    }
}

impl Heights {
    /// Forgets every height and every grid, as when the terrain view changed.
    pub fn clear(&self) {
        for slot in self.slots.iter() {
            slot.set(EMPTY);
        }
        self.regrid();
    }

    /// Forgets which grid each chunk is drawn on, which changes as chunks stream: done every frame.
    pub fn regrid(&self) {
        for grid in self.grids.iter() {
            grid.set(NO_GRID);
        }
    }

    /// Returns the height of `terrain` at grid point `(i, j)` of the finest grid, read once.
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

    /// Returns the grid `chunks` draw over chart position `at` of `terrain`, asked once per chunk.
    fn grid(&self, terrain: &Terrain, chunks: &Chunks, at: Vec2) -> Option<Drawn> {
        let coord = (at / CHUNK_SIZE).floor().as_ivec2();
        let hash =
            (coord.x.wrapping_mul(73_856_093) ^ coord.y.wrapping_mul(19_349_663)) as u32 as usize;
        let slot = &self.grids[hash % GRIDS];
        let (key, grid) = slot.get();
        if key == coord {
            return grid;
        }
        let grid = chunks.drawn_at(terrain, at);
        slot.set((coord, grid));
        grid
    }

    /// Returns the height the triangles of `grid` draw at chart position `at` of `terrain`, from
    /// cached grid points. Every grid of the terrain meshes is a whole number of finest cells wide
    /// and starts on a finest grid point.
    fn on(&self, terrain: &Terrain, grid: Drawn, at: Vec2) -> f32 {
        let cell = (at - grid.origin) / grid.step;
        let (i, j) = (cell.x.floor(), cell.y.floor());
        let (u, v) = (cell.x - i, cell.y - j);
        let k = ((grid.step / STEP).round() as i32).max(1);
        let base = (grid.origin / STEP).round().as_ivec2() + IVec2::new(i as i32, j as i32) * k;
        let h = |a: i32, b: i32| self.point(terrain, base.x + a * k, base.y + b * k);
        if u + v <= 1.0 {
            h(0, 0) + u * (h(1, 0) - h(0, 0)) + v * (h(0, 1) - h(0, 0))
        } else {
            h(1, 1) + (1.0 - u) * (h(0, 1) - h(1, 1)) + (1.0 - v) * (h(1, 0) - h(1, 1))
        }
    }

    /// Returns the height the finest terrain mesh draws at chart position `(x, z)` of `terrain`:
    /// what `Terrain::mesh_height` returns, from cached grid points.
    pub fn mesh(&self, terrain: &Terrain, x: f32, z: f32) -> f32 {
        self.on(terrain, Drawn::FINE, Vec2::new(x, z))
    }

    /// Returns the height of the terrain mesh `chunks` draw over chart position `(x, z)` of `terrain`,
    /// at the level it is drawn at: the finest mesh where they draw none of this chart, or without them.
    pub fn drawn(&self, terrain: &Terrain, chunks: Option<&Chunks>, x: f32, z: f32) -> f32 {
        let at = Vec2::new(x, z);
        let grid = chunks.and_then(|c| self.grid(terrain, c, at));
        self.on(terrain, grid.unwrap_or(Drawn::FINE), at)
    }
}
