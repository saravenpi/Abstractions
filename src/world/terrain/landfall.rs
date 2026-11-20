//! The spawn on a large planet: the shores the drainage grid finds, searched one after the other when no
//! ground near the origin of a view takes a spawn.

use bevy::math::DVec3;

use super::Terrain;
use super::spawn::{GLOBE_REACH, RINGS, SPAWN_STEP};
use crate::world::globe::Chart;

impl Terrain {
    /// Returns the rings of the spawn search: all of them on the legacy plane, and on a globe the share of
    /// the planet radius the search may reach.
    pub(super) fn reach(&self) -> u32 {
        match self.chart() {
            Chart::Flat => RINGS,
            _ => RINGS.min((self.globe().radius() * GLOBE_REACH / f64::from(SPAWN_STEP)) as u32),
        }
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
}
