//! Ponds: small basins carved below sea level in low, gentle inland ground.

use bevy::prelude::*;

use super::Terrain;
use crate::world::islands::smooth;
use crate::world::{Biome, SEA_LEVEL};

/// Width of the grid cells that each hold at most one pond candidate.
const CELL: f32 = 110.0;

/// Distance from a pond center beyond which it never changes the ground.
const REACH: f32 = 38.0;

/// Distance from a candidate center at which the ground must be dry, gentle land.
const MARGIN: f32 = 36.0;

/// A pond: a basin around `center` with an irregular outline.
struct Pond {
    center: Vec2,
    radius: f32,
    floor: f32,
    phase: [f32; 2],
}

/// Returns a deterministic number in `[0, 1)` for a seed, a cell and a slot.
fn mix(seed: u64, cell: IVec2, slot: u32) -> f32 {
    let mut h = seed ^ (cell.x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (cell.y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= (slot as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((h ^ (h >> 31)) >> 40) as f32 / (1u64 << 24) as f32
}

/// Returns the chance that a cell of a biome holds a pond.
fn pond_chance(biome: Biome) -> f32 {
    match biome {
        Biome::Meadow | Biome::Forest | Biome::Cherry | Biome::Tropical => 0.8,
        Biome::Taiga => 0.2,
        Biome::Desert => 0.05,
    }
}

impl Pond {
    /// Returns how strongly the pond pulls the ground down at `p` and whether `p` is inside the basin.
    fn pull(&self, p: Vec2) -> (f32, bool) {
        let d = p - self.center;
        let a = d.y.atan2(d.x);
        let r = self.radius
            * (1.0
                + 0.18 * (2.0 * a + self.phase[0]).sin()
                + 0.12 * (3.0 * a + self.phase[1]).sin());
        (
            1.0 - smooth(0.3 * r, 1.15 * r, d.length()),
            d.length() < 1.15 * r,
        )
    }
}

impl Terrain {
    /// Returns the pond candidate of `cell` when it is within reach of `p` and sits on good ground.
    fn pond_in_cell(&self, cell: IVec2, p: Vec2) -> Option<Pond> {
        let at = |k| 0.15 + 0.7 * mix(self.seed, cell, k);
        let center = (cell.as_vec2() + Vec2::new(at(0), at(1))) * CELL;
        if center.distance(p) > REACH {
            return None;
        }
        if mix(self.seed, cell, 2) > pond_chance(self.biome(center.x, center.y)) {
            return None;
        }
        let ground = |q: Vec2| self.shape.elevation(q.x, q.y, &self.climate);
        let h = ground(center);
        let firm = |dir: Vec2| {
            (ground(center + dir * MARGIN) - h).abs() < 4.5 && ground(center + dir * MARGIN) > 0.5
        };
        let flat = [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
            .into_iter()
            .all(firm);
        (flat && (0.9..4.6).contains(&h)).then(|| Pond {
            center,
            radius: 11.0 + 13.0 * mix(self.seed, cell, 3),
            floor: SEA_LEVEL - 0.9 - 1.1 * mix(self.seed, cell, 4),
            phase: [mix(self.seed, cell, 5) * 6.3, mix(self.seed, cell, 6) * 6.3],
        })
    }

    /// Returns the ponds that may touch the point `(x, z)`.
    fn ponds_near(&self, x: f32, z: f32) -> impl Iterator<Item = Pond> {
        let p = Vec2::new(x, z);
        let cell = (p / CELL).floor().as_ivec2();
        (-1..=1)
            .flat_map(move |i| (-1..=1).map(move |j| cell + IVec2::new(i, j)))
            .filter_map(move |c| self.pond_in_cell(c, p))
    }

    /// Returns the ground height at `(x, z)` with the ponds carved into the uncarved height `h`.
    pub(super) fn carve(&self, x: f32, z: f32, h: f32) -> f32 {
        let p = Vec2::new(x, z);
        self.ponds_near(x, z)
            .fold(h, |h, pond| h.min(h + (pond.floor - h) * pond.pull(p).0))
    }

    /// Returns true when `(x, z)` lies inside a pond basin.
    pub(super) fn in_pond(&self, x: f32, z: f32) -> bool {
        let p = Vec2::new(x, z);
        self.ponds_near(x, z).any(|pond| pond.pull(p).1)
    }
}
