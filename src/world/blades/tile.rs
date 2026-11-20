//! One tile of blades: every blade root of a square of the chart, standing on the triangles of the finest
//! terrain mesh, in the tangent frame at the middle of the tile.
//!
//! A blade is one triangle whose three corners all sit at its root; the vertex shader grows it. Its first
//! uv names the corner, its second holds the turf coordinates of the ground under it, and its colour holds
//! its rank, which the shader keeps or drops it by with the distance, and a seed for its height and sway.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use super::tune::{DENSE, SPARSE, TILE};
use crate::universe::hash::mix;
use crate::world::mesh::{Frame, GRID_STEP, wrap};
use crate::world::scatter::Carpet;
use crate::world::{CHUNK_SIZE, SEA_LEVEL, Surface, Terrain};

/// Terrain cells along one side of a tile.
const CELLS: usize = (TILE / GRID_STEP) as usize;

/// Quarters of turf a blade can stand in, and the lift of the second uv `y` per quarter, as in `turf.wesl`.
const TURF: (f32, f32) = (4.0, 512.0);

/// The corners of a blade: left and right at the root, then the tip.
const CORNERS: [[f32; 2]; 3] = [[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]];

/// The ground of one tile: its corner, its heights at the grid points, and where the turf may grow.
pub struct Ground {
    /// Chart position of the low corner of the tile.
    corner: Vec2,
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

    /// Returns the height the finest terrain mesh draws at `u`, in cells from the corner of the tile: its
    /// two triangles per cell, split as the mesh splits them. None off open grass.
    fn height(&self, u: Vec2) -> Option<f32> {
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

/// The blades of one tile, ready to become a mesh.
#[derive(Default)]
pub struct Blades {
    /// Root of each corner, in the frame of the tile.
    positions: Vec<[f32; 3]>,
    /// Which corner each is.
    corners: Vec<[f32; 2]>,
    /// Turf coordinates of the ground under each.
    turf: Vec<[f32; 2]>,
    /// Rank and seed of its blade.
    marks: Vec<[f32; 4]>,
}

impl Blades {
    /// Returns how many blades there are.
    pub fn len(&self) -> usize {
        self.positions.len() / CORNERS.len()
    }

    /// Adds a blade whose root is `root` in the frame of the tile, over ground of turf coordinates `turf`,
    /// with its rank and seed in `mark`.
    fn push(&mut self, root: Vec3, turf: [f32; 2], mark: [f32; 4]) {
        self.positions.extend([root.to_array(); 3]);
        self.corners.extend(CORNERS);
        self.turf.extend([turf; 3]);
        self.marks.extend([mark; 3]);
    }

    /// Converts the blades into a mesh, or none when there is no blade.
    pub fn into_mesh(self) -> Option<Mesh> {
        (self.len() > 0).then(|| {
            let count = self.positions.len();
            Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; count])
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.corners)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, self.turf)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.marks)
        })
    }
}

/// Returns the turf coordinates of chart position `w`, where the turf grows `thick`: its wrapped chart
/// position, lifted by the band of its quarter of turf.
fn turf_uv(w: Vec2, thick: f32) -> [f32; 2] {
    let chunk = (w / CHUNK_SIZE).floor().as_ivec2();
    let [x, z] = wrap(chunk, w);
    [x, z + (thick.min(1.0) * TURF.0).round() * TURF.1]
}

/// Returns the unit value of bits 40 and up of `x`.
fn unit(x: u64) -> f32 {
    (x >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns the blades of tile `key` of `terrain` on `ground`, at the dense count when `full` and the sparse
/// one otherwise, in the tangent frame at its middle, with that frame. A blade stands where `carpet` grows
/// on open grass above the sea, and not where `clear` says the ground is taken.
pub fn grow(
    (terrain, ground): (&Terrain, &Ground),
    key: IVec2,
    carpet: &Carpet,
    full: bool,
    clear: &dyn Fn(DVec3) -> bool,
) -> (Blades, Frame) {
    let frame = Frame::at(terrain, ground.corner + Vec2::splat(TILE * 0.5));
    let most = TILE * TILE * DENSE;
    let count = (TILE * TILE * if full { DENSE } else { SPARSE }) as usize;
    let id = key + terrain.shift(TILE);
    let salt = mix(((id.x as u32 as u64) << 32) | id.y as u32 as u64);
    let mut blades = Blades::default();
    for n in 0..count {
        let h = mix(salt ^ (n as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let u = Vec2::new(unit(h), unit(mix(h)));
        let w = ground.corner + u * TILE;
        let thick = carpet.at(w);
        let Some(height) = ground
            .height(u * CELLS as f32)
            .filter(|h| *h > SEA_LEVEL && thick > 0.0)
        else {
            continue;
        };
        let planet = terrain.embed(w.x, height, w.y);
        if clear(planet) {
            continue;
        }
        let rank = (n as f32 + unit(mix(h ^ 1))) / most;
        let mark = [rank, unit(mix(h ^ 2)), 0.0, 1.0];
        blades.push(frame.local(planet), turf_uv(w, thick), mark);
    }
    (blades, frame)
}
