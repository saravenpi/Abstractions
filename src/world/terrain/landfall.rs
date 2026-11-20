//! The spawn on a large planet: the shores the drainage grid finds, searched one after the other when no
//! ground near the origin of a view takes a spawn.

use bevy::math::{DVec3, Vec2};

use super::Terrain;
use super::spawn::{GLOBE_REACH, RINGS, SPAWN_STEP, ring_points};
use crate::world::globe::Chart;

/// Points of a ring skipped between two the search for land looks at: with 16 m between rings this is
/// about 130 m along a ring, under the 170 m an island must span to take a spawn.
const SPARSE: usize = 4;

impl Terrain {
    /// Returns the rings of the spawn search: all of them on the legacy plane, and on a globe the share of
    /// the planet radius the search may reach.
    pub(super) fn reach(&self) -> u32 {
        match self.chart() {
            Chart::Flat => RINGS,
            _ => RINGS.min((self.globe().radius() * GLOBE_REACH / f64::from(SPAWN_STEP)) as u32),
        }
    }

    /// Returns true when the sea spreads farther than the search of this view reaches from its origin, which
    /// the drainage grid sees on a planet whose cells are wider than the reach: the search then has nothing to
    /// find there, and goes straight to the shores.
    pub(super) fn open_sea(&self) -> bool {
        let globe = self.globe();
        if self.chart() == Chart::Flat || !globe.has_ground() {
            return false;
        }
        let reach = f64::from(self.reach()) * f64::from(SPAWN_STEP);
        let from = self.embed(0.0, 0.0, 0.0).normalize();
        globe.drainage().sea_around(from, reach, globe.radius())
    }

    /// Returns the planet position of the spawn on the nearest shore of the drainage grid that takes one,
    /// or nothing on the legacy plane and on a planet whose shores take none.
    pub(super) fn landfall(&self) -> Option<DVec3> {
        if self.chart() == Chart::Flat {
            return None;
        }
        let globe = self.globe();
        let from = self.embed(0.0, 0.0, 0.0).normalize();
        let mut shores = globe.landfalls();
        shores.sort_by(|a, b| a.angle_between(from).total_cmp(&b.angle_between(from)));
        shores.into_iter().find_map(|dir| {
            let site = Terrain::near(globe, dir * globe.radius()).around(0.0, 0.0);
            let spot = site.best_spot(self.reach())?;
            Some(site.embed(spot.x, 0.0, spot.y))
        })
    }

    /// Returns true when one of the points of ring `k` taken at the sparse stride of the search for land
    /// lies on dry ground.
    pub(super) fn land_on_ring(&self, k: u32) -> bool {
        let sea = self.globe().sea_level();
        ring_points(k)
            .step_by(SPARSE)
            .any(|p| self.height(p.x, p.y) >= sea + 2.0)
    }

    /// Scores the points of ring `k`, and returns the best of them with the score that beats `bound`.
    pub(super) fn ring_best(
        &self,
        k: u32,
        bound: f32,
        fallback: &mut Option<Vec2>,
    ) -> Option<(f32, Vec2)> {
        ring_points(k)
            .filter_map(|p| self.candidate((p, k), bound, fallback))
            .min_by(|a, b| a.0.total_cmp(&b.0))
    }
}
