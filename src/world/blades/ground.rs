//! The ground of one tile of blades: the heights and the open grass at its grid points, and the surface the
//! finest terrain mesh draws between them.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{SLACK, TILE};
use crate::world::mesh::GRID_STEP;
use crate::world::{Surface, Terrain};

/// Terrain cells along one side of a tile.
pub const CELLS: usize = (TILE / GRID_STEP) as usize;

/// The ground of one tile: its corner, its heights at the grid points, and where the turf may grow.
pub struct Ground {
    /// Chart position of the low corner of the tile.
    pub(super) corner: Vec2,
    /// Heights above sea level at the grid points, the first index along x.
    heights: [[f32; CELLS + 1]; CELLS + 1],
    /// Whether each grid point lies on open grass.
    grass: [[bool; CELLS + 1]; CELLS + 1],
}

impl Ground {
    /// Reads the ground of tile `key` of `terrain`: one height and one surface per grid point.
    pub fn read(terrain: &Terrain, key: IVec2) -> Self {
        let corner = key.as_vec2() * TILE;
        let at = |i: usize, j: usize| corner + Vec2::new(i as f32, j as f32) * GRID_STEP;
        Self {
            corner,
            heights: std::array::from_fn(|i| {
                std::array::from_fn(|j| terrain.height(at(i, j).x, at(i, j).y))
            }),
            grass: std::array::from_fn(|i| {
                std::array::from_fn(|j| terrain.surface(at(i, j).x, at(i, j).y) == Surface::Grass)
            }),
        }
    }

    /// Returns a centre and a radius in metres of a sphere that holds every blade the tile can grow: the eight
    /// corners of its box between the lowest and the highest grid point, and [`SLACK`] over.
    pub fn sphere(&self, terrain: &Terrain) -> (DVec3, f64) {
        let flat = self.heights.iter().flatten();
        let (low, high) = flat.fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let at = |x: f32, y: f32, z: f32| {
            terrain.embed(self.corner.x + x * TILE, y, self.corner.y + z * TILE)
        };
        let centre = at(0.5, (low + high) * 0.5, 0.5);
        let corners =
            (0..8).map(|n| at((n & 1) as f32, [low, high][n >> 2], ((n >> 1) & 1) as f32));
        let reach = corners.map(|c| c.distance(centre)).fold(0.0, f64::max);
        (centre, reach + f64::from(SLACK))
    }

    /// Returns the height the finest terrain mesh draws at `u`, in cells from the corner of the tile: its
    /// two triangles per cell, split as the mesh splits them. None off open grass.
    pub fn height(&self, u: Vec2) -> Option<f32> {
        let (i, j) = ((u.x as usize).min(CELLS - 1), (u.y as usize).min(CELLS - 1));
        let (a, b) = (u.x - i as f32, u.y - j as f32);
        let (ri, rj) = (
            (u.x.round() as usize).min(CELLS),
            (u.y.round() as usize).min(CELLS),
        );
        if !self.grass[ri][rj] {
            return None;
        }
        let h = |x: usize, z: usize| self.heights[i + x][j + z];
        Some(match a + b <= 1.0 {
            true => h(0, 0) + a * (h(1, 0) - h(0, 0)) + b * (h(0, 1) - h(0, 0)),
            false => h(1, 1) + (1.0 - a) * (h(0, 1) - h(1, 1)) + (1.0 - b) * (h(1, 0) - h(1, 1)),
        })
    }
}
