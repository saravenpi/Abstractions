//! Where the robot stands, as the workshop reads it: the place a tool is used on, the planet that
//! decides the treatments, and the life that names the tissues.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::craft::build::Place;
use crate::life::Life;
use crate::space::Location;
use crate::universe::{PlanetSpec, Universe};
use crate::world::Terrain;

/// Earth gravity and sea-level air, used before a planet is known.
const HOME: Place = Place {
    gravity: 9.81,
    air: 1.225,
};

/// The planet underfoot.
#[derive(SystemParam)]
pub struct Here<'w> {
    /// The universe.
    universe: Option<Res<'w, Universe>>,
    /// Which planet.
    location: Option<Res<'w, Location>>,
    /// Its ground and life.
    terrain: Option<Res<'w, Terrain>>,
}

impl Here<'_> {
    /// Returns the planet, once the universe and the location are known.
    pub fn planet(&self) -> Option<PlanetSpec> {
        let (universe, location) = (self.universe.as_deref()?, self.location.as_deref()?);
        Some(universe.planet(location.body))
    }

    /// Returns the place a tool is used on here.
    pub fn place(&self) -> Place {
        self.planet().map_or(HOME, |planet| Place::of(&planet))
    }

    /// Returns the life underfoot and the terrain seed of its planet.
    pub fn life(&self) -> Option<(&Life, u64)> {
        let terrain = self.terrain.as_deref()?;
        Some((terrain.globe().life()?, terrain.seed()))
    }
}
