//! The spawn search: a gentle spot near a coast on a real island, in a temperate biome when possible.

use bevy::prelude::*;

use crate::world::globe::Chart;
use crate::world::{Biome, Surface, Terrain};

/// Spacing of the spawn search rings.
const SPAWN_STEP: f32 = 16.0;

/// Number of rings searched, which bounds the search radius.
const RINGS: u32 = 600;

/// Share of the planet radius the search may reach on a globe, which keeps it on the near side.
const GLOBE_REACH: f64 = 1.2;

/// Radii of the rings sampled to check that land surrounds the spawn.
const ISLAND_RADII: [f32; 2] = [45.0, 85.0];

/// Share of the sampled ring points that must be dry for the spot to count as a real island.
const ISLAND_SHARE: f32 = 0.85;

/// Points sampled on each island ring.
const ISLAND_SAMPLES: u32 = 12;

impl Terrain {
    /// Returns true when `(x, z)` is gentle dry ground of grass or sand with dry neighbours.
    fn good_spawn(&self, x: f32, z: f32) -> bool {
        let sea = self.globe().sea_level();
        let dry = |dx: f32, dz: f32| self.height(x + dx, z + dz) > sea + 1.5;
        matches!(self.surface(x, z), Surface::Grass | Surface::Sand)
            && self.height(x, z) >= sea + 2.0
            && (self.height(x + 6.0, z) - self.height(x - 6.0, z)).abs() < 2.5
            && (self.height(x, z + 6.0) - self.height(x, z - 6.0)).abs() < 2.5
            && [(6.0, 0.0), (-6.0, 0.0), (0.0, 6.0), (0.0, -6.0)]
                .iter()
                .all(|&(dx, dz)| dry(dx, dz))
    }

    /// Returns the heights sampled on a ring of radius `r` around `(x, z)`.
    fn ring_heights(&self, x: f32, z: f32, r: f32) -> impl Iterator<Item = f32> {
        (0..ISLAND_SAMPLES).map(move |a| {
            let d = Vec2::from_angle(a as f32 * std::f32::consts::TAU / ISLAND_SAMPLES as f32) * r;
            self.height(x + d.x, z + d.y)
        })
    }

    /// Returns true when land surrounds `(x, z)` out to about two hectares.
    fn on_island(&self, x: f32, z: f32) -> bool {
        let total = (ISLAND_RADII.len() as u32 * ISLAND_SAMPLES) as f32;
        let sea = self.globe().sea_level();
        let dry = ISLAND_RADII
            .iter()
            .flat_map(|&r| self.ring_heights(x, z, r))
            .filter(|&h| h > sea)
            .count();
        dry as f32 / total >= ISLAND_SHARE
    }

    /// Returns a penalty in metres for a spawn that is inland, high or in an harsh biome.
    fn spawn_penalty(&self, x: f32, z: f32) -> f32 {
        let harsh = matches!(self.biome(x, z), Biome::Desert | Biome::Taiga);
        let sea = self.globe().sea_level();
        let coastal = self.ring_heights(x, z, 70.0).any(|h| h <= sea);
        let high = self.height(x, z) > 10.0;
        300.0 * f32::from(harsh) + 200.0 * f32::from(!coastal) + 100.0 * f32::from(high)
    }

    /// Returns the score of an island spot on ring `k`: its distance plus its penalty.
    fn island_score(&self, p: Vec2, k: u32) -> Option<(f32, Vec2)> {
        self.on_island(p.x, p.y)
            .then(|| (k as f32 * SPAWN_STEP + self.spawn_penalty(p.x, p.y), p))
    }

    /// Returns the best spot within `rings` rings of the origin of this metric view: a candidate
    /// scored by distance plus its penalty.
    fn best_spot(&self, rings: u32) -> Vec2 {
        let mut best: Option<(f32, Vec2)> = None;
        let mut fallback: Option<Vec2> = None;
        for k in 0..rings {
            if best.is_some_and(|(score, _)| k as f32 * SPAWN_STEP > score) {
                break;
            }
            let found = ring_points(k)
                .filter(|p| self.good_spawn(p.x, p.y))
                .inspect(|p| {
                    fallback.get_or_insert(*p);
                })
                .filter_map(|p| self.island_score(p, k))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if found.is_some_and(|f| best.is_none_or(|b| f.0 < b.0)) {
                best = found;
            }
        }
        best.map(|(_, p)| p).or(fallback).unwrap_or(Vec2::ZERO)
    }

    /// Returns the best spawn near the origin of the chart, in chart coordinates.
    ///
    /// The search runs in metres on every chart: a globe view is searched through its metric view
    /// around the origin, so distances and slopes mean the same as on the plane.
    pub fn find_spawn(&self) -> Vec3 {
        let site = self.around(0.0, 0.0);
        let rings = match self.chart() {
            Chart::Flat => RINGS,
            _ => RINGS.min((self.globe().radius() * GLOBE_REACH / f64::from(SPAWN_STEP)) as u32),
        };
        let spot = site.best_spot(rings);
        let at = self.locate(site.embed(spot.x, 0.0, spot.y));
        Vec3::new(at.x, self.height(at.x, at.z), at.z)
    }
}

/// Returns the points of search ring `k`, evenly spaced.
fn ring_points(k: u32) -> impl Iterator<Item = Vec2> {
    let n = (k * 6).max(1);
    (0..n).map(move |a| {
        Vec2::from_angle(a as f32 * std::f32::consts::TAU / n as f32) * (k as f32 * SPAWN_STEP)
    })
}
