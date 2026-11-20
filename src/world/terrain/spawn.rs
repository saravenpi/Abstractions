//! The spawn search: a gentle spot near a coast on a real island, in a temperate biome on grass or sand when
//! possible, on flat snow or rock near the poles where nothing else lies.

use bevy::prelude::*;

use crate::world::globe::Chart;
use crate::world::{Biome, Surface, Terrain};

/// Spacing of the spawn search rings.
pub(super) const SPAWN_STEP: f32 = 16.0;

/// Number of rings searched, which bounds the search radius.
pub(super) const RINGS: u32 = 600;

/// Share of the planet radius the search may reach on a globe, which keeps it on the near side.
pub(super) const GLOBE_REACH: f64 = 1.2;

/// Radii of the rings sampled to check that land surrounds the spawn.
const ISLAND_RADII: [f32; 2] = [45.0, 85.0];

/// Share of the sampled ring points that must be dry for the spot to count as a real island.
const ISLAND_SHARE: f32 = 0.85;

/// Points sampled on each island ring.
const ISLAND_SAMPLES: u32 = 12;

impl Terrain {
    /// Returns true when `(x, z)` is gentle dry ground with dry neighbours out of any river: grass or sand,
    /// or on a sphere the snow and the rock of the cold and the high land, which the slope limit keeps
    /// flat. The legacy plane keeps to grass and sand, as it always did.
    pub(super) fn good_spawn(&self, x: f32, z: f32) -> bool {
        let sea = self.globe().sea_level();
        let dry = |dx: f32, dz: f32| {
            self.height(x + dx, z + dz) > sea + 1.5 && !self.river_at(x + dx, z + dz)
        };
        let stands = || match self.surface(x, z) {
            Surface::Grass | Surface::Sand => true,
            Surface::Snow | Surface::Rock => self.chart() != Chart::Flat,
            _ => false,
        };
        self.height(x, z) >= sea + 2.0
            && (self.height(x + 6.0, z) - self.height(x - 6.0, z)).abs() < 2.5
            && (self.height(x, z + 6.0) - self.height(x, z - 6.0)).abs() < 2.5
            && stands()
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

    /// Returns a penalty in metres for a spawn that is high, in an harsh biome or on snow or rock, so that
    /// grass or sand near by wins.
    fn rough_penalty(&self, x: f32, z: f32) -> f32 {
        let bare = || matches!(self.surface(x, z), Surface::Snow | Surface::Rock);
        let harsh = matches!(self.biome(x, z), Biome::Desert | Biome::Taiga) || bare();
        300.0 * f32::from(harsh) + 100.0 * f32::from(self.height(x, z) > 10.0)
    }

    /// Returns a penalty in metres for a spawn that is inland.
    fn inland_penalty(&self, x: f32, z: f32) -> f32 {
        let sea = self.globe().sea_level();
        200.0 * f32::from(!self.ring_heights(x, z, 70.0).any(|h| h <= sea))
    }

    /// Returns the score of the spawn at `p` on ring `k`, which is its distance plus its penalty, when it is a
    /// good spawn on an island and the score beats `bound`. Once a spot is known, the distance and the rough
    /// penalty are read before anything else, since they prune most of what is left. The first good spawn is
    /// noted in `fallback`.
    fn candidate(
        &self,
        (p, k): (Vec2, u32),
        bound: f32,
        fallback: &mut Option<Vec2>,
    ) -> Option<(f32, Vec2)> {
        let reach = || k as f32 * SPAWN_STEP + self.rough_penalty(p.x, p.y);
        let early = bound.is_finite().then(reach);
        if early.is_some_and(|early| early >= bound) || !self.good_spawn(p.x, p.y) {
            return None;
        }
        fallback.get_or_insert(p);
        let score = early.unwrap_or_else(reach) + self.inland_penalty(p.x, p.y);
        (score < bound && self.on_island(p.x, p.y)).then_some((score, p))
    }

    /// Returns the best spot within `rings` rings of the origin of this metric view: a candidate
    /// scored by distance plus its penalty, or nothing when no ground there takes a spawn.
    pub(super) fn best_spot(&self, rings: u32) -> Option<Vec2> {
        let mut best: Option<(f32, Vec2)> = None;
        let mut fallback: Option<Vec2> = None;
        for k in 0..rings {
            let bound = best.map_or(f32::INFINITY, |(score, _)| score);
            if k as f32 * SPAWN_STEP > bound {
                break;
            }
            let found = ring_points(k)
                .filter_map(|p| self.candidate((p, k), bound, &mut fallback))
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if found.is_some() {
                best = found;
            }
        }
        best.map(|(_, p)| p).or(fallback)
    }

    /// Returns the best spawn near the origin of the chart, in chart coordinates.
    ///
    /// The search runs in metres on every chart: a globe view is searched through its metric view
    /// around the origin, so distances and slopes mean the same as on the plane. Where no ground near
    /// the origin takes a spawn, which is open sea on a large planet, the search goes on from the shores
    /// of the drainage grid, see [`Terrain::landfall`].
    pub fn find_spawn(&self) -> Vec3 {
        self.spawn().unwrap_or_else(|| {
            let at = self.locate(self.around(0.0, 0.0).embed(0.0, 0.0, 0.0));
            Vec3::new(at.x, self.height(at.x, at.z), at.z)
        })
    }

    /// Returns the best spawn near the origin of the chart, or on a shore of the drainage grid when none is
    /// near, or nothing when no ground takes a spawn.
    pub fn spawn(&self) -> Option<Vec3> {
        let site = self.around(0.0, 0.0);
        let spot = (!self.open_sea())
            .then(|| site.best_spot(self.reach()))
            .flatten()
            .map(|s| site.embed(s.x, 0.0, s.y));
        let at = self.locate(spot.or_else(|| self.landfall())?);
        Some(Vec3::new(at.x, self.height(at.x, at.z), at.z))
    }
}

/// Returns the points of search ring `k`, evenly spaced.
fn ring_points(k: u32) -> impl Iterator<Item = Vec2> {
    let n = (k * 6).max(1);
    (0..n).map(move |a| {
        Vec2::from_angle(a as f32 * std::f32::consts::TAU / n as f32) * (k as f32 * SPAWN_STEP)
    })
}
