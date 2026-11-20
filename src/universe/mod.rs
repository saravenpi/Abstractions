//! The universe as data: galaxies, stars and planets, each a pure function of one seed.
//!
//! Everything here is 64-bit and free of game systems. Distances are metres, times are seconds.

mod air;
mod class;
mod climate;
mod density;
mod galaxy;
mod giant;
pub mod hash;
mod hill;
mod home;
mod ids;
mod look;
mod moon;
mod names;
mod orbit;
mod planet;
mod record;
mod rings;
mod sector;
mod spec;
mod spin;
mod star;
mod system;
mod units;
mod volatiles;
mod zone;

#[cfg(test)]
mod class_tests;
#[cfg(test)]
mod climate_tests;
#[cfg(test)]
mod giant_tests;
#[cfg(test)]
mod home_tests;
#[cfg(test)]
mod moon_tests;
#[cfg(test)]
mod range_moon_tests;
#[cfg(test)]
mod range_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod ring_tests;
#[cfg(test)]
mod spin_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod volatiles_tests;

pub use air::{Gas, Mix, NO_AIR, Part, TOP_SCALES, dense, heights};
pub use class::{MOLTEN, VOLCANIC};
pub use galaxy::{ARMS, CENTER_SPAN, RADIUS_GLY};
pub use giant::{GIANT_MASS, GIANT_RADIUS};
pub use hill::{CAPTURED_RADIUS, IMPACT_RADIUS, REGULAR_RADIUS};
pub use home::landable;
pub use ids::{BodyId, GalaxyId, StarId};
pub use look::Look;
pub use planet::{PLANET_RADIUS, ROCK, gravity, orbit_seconds};
pub use record::{PlanetClass, PlanetRecord, Water, Zone};
pub use rings::{RING_OUTER, ROCHE};
pub use sector::{SECTOR_STARS, sector_of};
pub use spec::{
    GRAVITATION, GalaxyKind, GalaxySpec, Orbit, PlanetSpec, Rings, StarClass, StarSpec, SystemSpec,
};
pub use spin::breakup;
pub use system::{ORBIT, PLANETS};
pub use units::{
    AU, EARTH_RADIUS, STANDARD_GRAVITY, STARLIGHT, SUN_MASS, SUN_RADIUS, SUN_TEMPERATURE, YEAR,
};
pub use units::{EARTH_RADIUS as HOME_RADIUS, STANDARD_GRAVITY as HOME_GRAVITY};

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
    home: BodyId,
}

impl Universe {
    /// Builds the universe for a seed. The same seed always gives the same universe.
    ///
    /// This searches the middle of the home galaxy for the home planet, which takes a few
    /// milliseconds: keep the universe, do not build it again every frame.
    pub fn new(seed: u64) -> Self {
        let home = home::choose(seed);
        Self { seed, home }
    }

    /// Returns the seed this universe was built from.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the planet a new game starts on, always the same one for a seed: the nearest
    /// planet to the centre of the home galaxy that is solid, landable and alive. It is found
    /// once, when the universe is built.
    pub fn home(&self) -> BodyId {
        self.home
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
        let star = self.star(id).or_else(|| self.star(self.home.star));
        let star = star.expect("the home galaxy always holds the home star");
        SystemSpec {
            planets: system::planets(&star),
            star,
        }
    }

    /// Returns one planet or moon. An index past the last planet gives the outermost one, and a
    /// moon that does not exist gives its planet.
    pub fn planet(&self, id: BodyId) -> PlanetSpec {
        let SystemSpec { star, mut planets } = self.system(id.star);
        let last = planets.len() - 1;
        let planet = planets.swap_remove(usize::from(id.index).min(last));
        moon::ranked(&star, &planet, id.moon).unwrap_or(planet)
    }
}
