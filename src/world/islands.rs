//! Island placement: at most one island per square cell, derived from the world seed.

use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Side length of one island cell in world units.
pub const CELL: f32 = 256.0;

/// Height of the open sea floor.
pub const SEA_FLOOR: f32 = -12.0;

/// Chance that a cell holds an island.
const ISLAND_CHANCE: f64 = 0.65;

/// A single island: a disc where the land rises out of the sea floor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Island {
    /// World position of the island center on the XZ plane.
    pub center: Vec2,
    /// Radius where the land meets the sea floor.
    pub radius: f32,
    /// Height of the island center above sea level, before noise.
    pub peak: f32,
}

impl Island {
    /// Returns the land rise above the sea floor at world position `(x, z)`.
    pub fn rise(&self, x: f32, z: f32) -> f32 {
        let d = Vec2::new(x, z).distance(self.center) / self.radius;
        if d >= 1.0 {
            return 0.0;
        }
        let f = 1.0 - d * d;
        f * f * (self.peak - SEA_FLOOR)
    }

    /// Returns how strongly the island shapes the ground at `(x, z)`, from 0 to 1.
    pub fn influence(&self, x: f32, z: f32) -> f32 {
        let d = Vec2::new(x, z).distance(self.center) / self.radius;
        (1.0 - d * d).clamp(0.0, 1.0)
    }
}

/// Mixes the seed and a cell coordinate into one well spread 64-bit value.
fn cell_hash(seed: u64, cx: i32, cz: i32) -> u64 {
    let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
    for v in [cx as u32 as u64, cz as u32 as u64] {
        h = (h ^ v).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 31;
    }
    h
}

/// Returns the island of cell `(cx, cz)`, if the seed puts one there.
pub fn island_in_cell(seed: u64, cx: i32, cz: i32) -> Option<Island> {
    let mut rng = ChaCha8Rng::seed_from_u64(cell_hash(seed, cx, cz));
    if !rng.random_bool(ISLAND_CHANCE) {
        return None;
    }
    let radius = rng.random_range(40.0..110.0);
    let jitter = Vec2::new(
        rng.random_range(40.0..CELL - 40.0),
        rng.random_range(40.0..CELL - 40.0),
    );
    let t = (radius - 40.0) / 70.0;
    let peak = 8.0 + t * 14.0 + rng.random_range(0.0..3.0);
    Some(Island {
        center: Vec2::new(cx as f32, cz as f32) * CELL + jitter,
        radius,
        peak,
    })
}

/// Returns the island cell holding world position `(x, z)`.
pub fn cell_of(x: f32, z: f32) -> (i32, i32) {
    ((x / CELL).floor() as i32, (z / CELL).floor() as i32)
}

/// Returns the islands of the 3x3 cells around world position `(x, z)`.
pub fn islands_near(seed: u64, x: f32, z: f32) -> impl Iterator<Item = Island> {
    let (cx, cz) = cell_of(x, z);
    (-1..=1)
        .flat_map(move |dx| (-1..=1).filter_map(move |dz| island_in_cell(seed, cx + dx, cz + dz)))
}
