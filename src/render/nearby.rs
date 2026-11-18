//! The star system around the camera: its star, its planets and their moons, read once per system.

use bevy::math::DVec3;

use super::moons::eclipse;
use super::vantage::Vantage;
use crate::universe::{BodyId, PlanetSpec, StarId, SystemSpec, Universe};
use crate::world::globe::far::handover;

/// The star system around the camera, read from the universe once per system.
#[derive(Debug, Clone, PartialEq)]
pub struct Nearby {
    /// Star the system was asked for.
    pub id: StarId,
    /// The star and its planets.
    pub system: SystemSpec,
    /// Radius of the star disc of the galaxy in metres.
    pub galaxy_radius: f64,
    /// The moons of every planet, planet after planet and outward around each one.
    pub moons: Vec<PlanetSpec>,
}

impl Nearby {
    /// Reads the system of a star, the moons of its planets and the size of its galaxy from the universe.
    pub fn read(universe: &Universe, id: StarId) -> Self {
        let system = universe.system(id);
        let moons = system
            .planets
            .iter()
            .flat_map(|planet| universe.moons(planet.id))
            .collect();
        Self {
            id,
            system,
            galaxy_radius: universe.galaxy(id.galaxy).map_or(1.0, |g| g.radius),
            moons,
        }
    }

    /// Returns planet number `index`, or the outermost one past the end.
    pub fn planet(&self, index: u8) -> Option<&PlanetSpec> {
        let planets = &self.system.planets;
        planets.get(usize::from(index)).or(planets.last())
    }

    /// Returns moon number `moon` of planet number `index`: the planet itself for 0 and for a
    /// moon that does not exist, as the universe does.
    pub fn body(&self, index: u8, moon: u8) -> Option<&PlanetSpec> {
        let planet = self.planet(index)?;
        let own = |body: &&PlanetSpec| body.id.index == planet.id.index && body.id.moon == moon;
        Some(self.moons.iter().find(own).unwrap_or(planet))
    }

    /// Returns the planet or moon `id`, as [`Nearby::body`] does.
    pub fn find(&self, id: BodyId) -> Option<&PlanetSpec> {
        self.body(id.index, id.moon)
    }

    /// Returns the share of starlight that reaches the point `at`, metres from the star, past
    /// the planet of `own` and its moons at `time`, the globe of `own` itself left out.
    pub fn eclipse(&self, at: DVec3, own: BodyId, time: f64) -> f32 {
        eclipse(at, self.family(own.index), own, time)
    }

    /// Returns the metres from the centre of the body under the camera of `vantage` past which
    /// the sky draws that body, clouds included, instead of the world.
    pub fn handover(&self, vantage: &Vantage) -> f64 {
        let body = u8::try_from(vantage.body)
            .ok()
            .and_then(|index| self.body(index, vantage.moon));
        body.map_or(f64::MAX, |body| handover(body.radius, body.rings))
    }

    /// Returns every planet, then every moon.
    pub fn bodies(&self) -> impl Iterator<Item = &PlanetSpec> {
        self.system.planets.iter().chain(&self.moons)
    }

    /// Returns planet number `index` and its moons: the bodies that light and shade each other.
    pub fn family(&self, index: u8) -> impl Iterator<Item = &PlanetSpec> {
        self.bodies().filter(move |body| body.id.index == index)
    }
}
