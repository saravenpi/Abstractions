//! The terrain height field: the single source of truth for where land is.

use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::islands::{CELL, Island, SEA_FLOOR, cell_of, island_in_cell, islands_near};
use super::surface::{Surface, classify};

/// Finite difference step used to measure slopes.
const SLOPE_STEP: f32 = 1.5;

/// Deterministic infinite terrain built from a seed.
#[derive(Resource, Clone)]
pub struct Terrain {
    seed: u64,
    noise: Fbm<Perlin>,
}

impl Terrain {
    /// Builds the terrain for a seed. The same seed always gives the same world.
    pub fn new(seed: u64) -> Self {
        let noise = Fbm::<Perlin>::new(seed as u32 ^ (seed >> 32) as u32).set_octaves(4);
        Self { seed, noise }
    }

    /// Returns the seed this terrain was built from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the ground height at world position `(x, z)`.
    pub fn height(&self, x: f32, z: f32) -> f32 {
        let n = self.noise.get([x as f64 * 0.02, z as f64 * 0.02]) as f32;
        let best = islands_near(self.seed, x, z)
            .map(|i| (i.rise(x, z), i))
            .max_by(|a, b| a.0.total_cmp(&b.0));
        let Some((rise, island)) = best else {
            return SEA_FLOOR + n * 0.8;
        };
        SEA_FLOOR + rise + n * (0.8 + island.peak * 0.25 * island.influence(x, z))
    }

    /// Returns what covers the ground at world position `(x, z)`.
    pub fn surface(&self, x: f32, z: f32) -> Surface {
        let h = self.height(x, z);
        let e = SLOPE_STEP;
        let dx = (self.height(x + e, z) - self.height(x - e, z)) / (2.0 * e);
        let dz = (self.height(x, z + e) - self.height(x, z - e)) / (2.0 * e);
        classify(h, dx.hypot(dz))
    }

    /// Returns a land point on a random island chosen from the seed, with `y` at ground height.
    pub fn find_spawn(&self) -> Vec3 {
        let island = self.spawn_island();
        let at = |p: Vec2| Vec3::new(p.x, self.height(p.x, p.y), p.y);
        let good = |p: Vec2| self.surface(p.x, p.y) == Surface::Grass;
        let center = island.center;
        let ring = (1..=8).flat_map(|r| {
            (0..16).map(move |a| {
                let angle = a as f32 * std::f32::consts::TAU / 16.0;
                center + Vec2::from_angle(angle) * (r as f32 * island.radius * 0.06)
            })
        });
        let spot = std::iter::once(center).chain(ring).find(|p| good(*p));
        at(spot.unwrap_or(center))
    }

    /// Picks the island nearest to the origin, choosing among ties of the same ring from the seed.
    fn spawn_island(&self) -> Island {
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed ^ 0x5EED);
        let (ox, oz) = cell_of(0.0, 0.0);
        for r in 0..=32i32 {
            let found: Vec<Island> = (-r..=r)
                .flat_map(|dx| (-r..=r).map(move |dz| (dx, dz)))
                .filter(|(dx, dz)| dx.abs().max(dz.abs()) == r)
                .filter_map(|(dx, dz)| island_in_cell(self.seed, ox + dx, oz + dz))
                .collect();
            if !found.is_empty() {
                return found[rng.random_range(0..found.len())];
            }
        }
        Island {
            center: Vec2::splat(CELL * 0.5),
            radius: 60.0,
            peak: 12.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::SEA_LEVEL;

    /// The same seed gives the same heights and a different seed does not.
    #[test]
    fn heights_are_deterministic() {
        let (a, b, c) = (Terrain::new(7), Terrain::new(7), Terrain::new(8));
        let pts = [(0.0, 0.0), (130.5, -77.2), (-900.0, 412.0)];
        assert!(pts.iter().all(|&(x, z)| a.height(x, z) == b.height(x, z)));
        assert!(pts.iter().any(|&(x, z)| a.height(x, z) != c.height(x, z)));
    }

    /// Every spawn is on dry land.
    #[test]
    fn spawn_is_always_land() {
        for seed in 0..50 {
            let t = Terrain::new(seed);
            let p = t.find_spawn();
            assert!(p.y > SEA_LEVEL + 1.0, "seed {seed} spawn {p}");
            assert_eq!(p, Terrain::new(seed).find_spawn());
        }
    }
}
