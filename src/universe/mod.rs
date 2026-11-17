//! The universe as data: galaxies, stars and planets, each a pure function of one seed.
//!
//! Everything here is 64-bit and free of game systems. Distances are metres, times are seconds.

mod density;
mod galaxy;
mod hash;
mod home;
mod ids;
mod names;
mod orbit;
mod planet;
mod sector;
mod spec;
mod star;
mod system;

#[cfg(test)]
mod home_tests;
#[cfg(test)]
mod range_tests;
#[cfg(test)]
mod tests;

pub use galaxy::{ARMS, CENTER_SPAN, RADIUS_GLY};
pub use home::{HOME_DAY_SECONDS, HOME_GRAVITY, HOME_RADIUS};
pub use ids::{BodyId, GalaxyId, StarId};
pub use names::{HOME_PLANET_NAME, HOME_STAR_NAME};
pub use planet::{PLANET_DAY, PLANET_GRAVITY, PLANET_RADIUS, air, orbit_seconds};
pub use sector::{SECTOR_STARS, sector_of};
pub use spec::{GRAVITATION, GalaxyKind, GalaxySpec, PlanetSpec, StarClass, StarSpec, SystemSpec};
pub use system::{ORBIT, PLANETS};

use bevy::math::IVec3;
use bevy::prelude::Resource;

/// Metres in one game light-year.
pub const GLY: f64 = 1e10;

/// Width of one star sector in game light-years.
pub const SECTOR_GLY: f64 = 10.0;

/// Width in metres of the grid cells that each hold at most one galaxy.
pub const GALAXY_CELL: f64 = 1e15;

/// The whole universe, generated on demand from a seed.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Universe {
    seed: u64,
}

impl Universe {
    /// Builds the universe for a seed. The same seed always gives the same universe.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Returns the seed this universe was built from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the planet a new game starts on, always the same one for a seed.
    pub fn home(&self) -> BodyId {
        home::body(self.seed)
    }

    /// Returns the galaxy of a grid cell, or nothing when the cell is empty.
    pub fn galaxy(&self, id: GalaxyId) -> Option<GalaxySpec> {
        galaxy::spec(self.seed, id)
    }

    /// Returns the galaxies whose cell lies within `cells` cells of `id` on every axis, `id` included.
    pub fn galaxies_near(&self, id: GalaxyId, cells: i32) -> Vec<GalaxySpec> {
        let n = 2 * cells.max(0) + 1;
        let offsets = (0..n * n * n).map(|i| IVec3::new(i % n, i / n % n, i / (n * n)) - cells);
        offsets
            .filter_map(|d| self.galaxy(GalaxyId(id.0 + d)))
            .collect()
    }

    /// Returns the stars of one sector of a galaxy, `stars[i].id.index == i`.
    pub fn stars_in_sector(&self, galaxy: GalaxyId, sector: IVec3) -> Vec<StarSpec> {
        let mut stars = Vec::new();
        if let Some(shape) = galaxy::shape(self.seed, galaxy) {
            sector::fill(&shape, galaxy, sector, &mut stars);
        }
        stars
    }

    /// Returns a star with its planets. A star that does not exist gives the home system.
    pub fn system(&self, id: StarId) -> SystemSpec {
        let home = self.home().star;
        let star = self.star(id).or_else(|| self.star(home));
        let star = star.expect("the home sector always holds the home star");
        let world = (star.id == home).then_some(self.seed);
        SystemSpec {
            planets: system::planets(&star, world),
            star,
        }
    }

    /// Returns one planet. An index past the last planet gives the outermost one.
    pub fn planet(&self, id: BodyId) -> PlanetSpec {
        let mut planets = self.system(id.star).planets;
        let last = planets.len() - 1;
        planets.swap_remove(usize::from(id.index).min(last))
    }
}
