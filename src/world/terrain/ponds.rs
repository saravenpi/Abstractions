//! Ponds: small basins carved below sea level in low, gentle inland ground.

use bevy::prelude::*;

use crate::world::globe::{Globe, Site};
use crate::world::islands::smooth;
use crate::world::{Biome, SEA_LEVEL};

/// Width of the grid cells that hold one pond candidate each on average, at most one on the plane.
const CELL: f32 = 110.0;

/// Distance from a pond center beyond which it never changes the ground.
const REACH: f32 = 38.0;

/// Distance from a candidate center at which the ground must be dry, gentle land.
const MARGIN: f32 = 36.0;

/// Ground height at or below which no pond floor can lower the ground: every floor is above -2 m.
const BELOW_PONDS: f32 = SEA_LEVEL - 2.1;

/// A pond: a basin around `center` with an irregular outline.
struct Pond<S> {
    center: S,
    radius: f32,
    floor: f32,
    phase: [f32; 2],
}

/// Returns a deterministic number in `[0, 1)` for a seed, a cell and a slot.
fn mix(seed: u64, cell: IVec3, slot: u32) -> f32 {
    let mut h = seed ^ (cell.x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= (cell.y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= (cell.z as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93);
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

impl<S: Site> Pond<S> {
    /// Returns how strongly the pond pulls the ground down at `p` and whether `p` is inside the basin.
    fn pull(&self, p: S) -> (f32, bool) {
        let d = p.offset(self.center);
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

impl Globe {
    /// Returns the pond candidate of `cell` when it is within reach of `p` and sits on good ground.
    fn pond_in_cell<S: Site>(&self, cell: IVec3, p: S) -> Option<Pond<S>> {
        let center = p.anchor(cell, CELL, |slot| 0.15 + 0.7 * mix(self.seed, cell, slot))?;
        if p.offset(center).length() > REACH {
            return None;
        }
        if mix(self.seed, cell, 2) > pond_chance(self.biome_on(center)) {
            return None;
        }
        let ground = |q: S| self.uncarved(q);
        let h = ground(center);
        let firm = |dir: Vec2| {
            let rim = ground(center.moved(dir * MARGIN));
            (rim - h).abs() < 4.5 && rim > 0.5
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

    /// Returns the ponds that may touch the site `p`.
    fn ponds_near<S: Site>(&self, p: S) -> impl Iterator<Item = Pond<S>> {
        p.cells(CELL).filter_map(move |c| self.pond_in_cell(c, p))
    }

    /// Returns the ground height at `p` with the ponds carved into the uncarved height `h`.
    pub(in crate::world) fn carve<S: Site>(&self, p: S, h: f32) -> f32 {
        if h <= BELOW_PONDS || !self.lands().ponds() {
            return h;
        }
        self.ponds_near(p)
            .fold(h, |h, pond| h.min(h + (pond.floor - h) * pond.pull(p).0))
    }

    /// Returns true when `p` lies inside a pond basin.
    pub(in crate::world) fn in_pond<S: Site>(&self, p: S) -> bool {
        self.lands().ponds() && self.ponds_near(p).any(|pond| pond.pull(p).1)
    }
}
