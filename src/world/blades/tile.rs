//! One tile of blades: every blade root of a square of the chart, standing on the triangles of the finest
//! terrain mesh, in the tangent frame at the middle of the tile.
//!
//! A blade is one triangle whose three corners all sit at its root; the vertex shader grows it. Its first
//! uv names the corner, its second holds the turf coordinates of the ground under it, and its colour holds
//! its rank, which the shader keeps or drops it by with the distance, a seed for its height and sway, and
//! the share of the second species and the speckle of the carpet under it, baked once per blade.

use bevy::asset::RenderAssetUsages;
use bevy::math::DVec3;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use super::colour::tint;
use super::ground::{CELLS, Ground};
use super::tune::{DENSE, SPARSE, TILE};
use crate::universe::hash::mix;
use crate::world::mesh::{Frame, wrap};
use crate::world::scatter::Carpet;
use crate::world::{CHUNK_SIZE, SEA_LEVEL, Terrain};

/// Quarters of turf a blade can stand in, and the lift of the second uv `y` per quarter, as in `turf.wesl`.
const TURF: (f32, f32) = (4.0, 512.0);

/// The corners of a blade: left and right at the root, then the tip.
const CORNERS: [[f32; 2]; 3] = [[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]];

/// The blades of one tile, ready to become a mesh.
#[derive(Default)]
pub struct Blades {
    /// Root of each corner, in the frame of the tile.
    positions: Vec<[f32; 3]>,
    /// Which corner each is.
    corners: Vec<[f32; 2]>,
    /// Turf coordinates of the ground under each.
    turf: Vec<[f32; 2]>,
    /// Rank, seed, share of the second species and speckle of its blade.
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

    /// Returns the root of every corner of every blade, in the frame of the tile.
    #[cfg(test)]
    pub fn roots(&self) -> &[[f32; 3]] {
        &self.positions
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
/// position, lifted by the band of its quarter of turf, and the wrapped position alone.
fn turf_uv(w: Vec2, thick: f32) -> ([f32; 2], Vec2) {
    let chunk = (w / CHUNK_SIZE).floor().as_ivec2();
    let [x, z] = wrap(chunk, w);
    (
        [x, z + (thick.min(1.0) * TURF.0).round() * TURF.1],
        Vec2::new(x, z),
    )
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
        let (uv, spot) = turf_uv(w, thick);
        let (share, speckle) = tint(spot);
        let mark = [rank, unit(mix(h ^ 2)), share, speckle];
        blades.push(frame.local(planet), uv, mark);
    }
    (blades, frame)
}
