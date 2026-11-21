//! Shared setup of the tests of the orbital cruise: a planet of the size, the gravity and the air
//! of the Earth, and a cruise started on its equator.

use bevy::math::DVec3;

use super::skim::{Band, Skim};
use crate::universe::{EARTH_RADIUS, PlanetSpec, STANDARD_GRAVITY, Universe};

/// Seconds of one step of the cruise in the tests: a frame at 60 frames per second.
pub const DT: f64 = 1.0 / 60.0;

/// Returns a planet of the size and the gravity of the Earth with its air ending at 101 km, as
/// twelve scale heights of 8.4 km do.
pub fn earth() -> PlanetSpec {
    let universe = Universe::new(7);
    PlanetSpec {
        radius: EARTH_RADIUS,
        gravity: STANDARD_GRAVITY,
        atmosphere_height: 1.01e5,
        ..universe.planet(universe.home())
    }
}

/// Returns a cruise at rest `height` metres over the equator of `planet` at +X, heading east.
pub fn skim_at(planet: &PlanetSpec, height: f64) -> Skim {
    Skim {
        pos: DVec3::X * (planet.radius + height),
        heading: DVec3::NEG_Z,
        speed: 0.0,
        height,
    }
}

/// Returns the band of `planet`, checked to start above its air.
pub fn band(planet: &PlanetSpec) -> Band {
    let band = Band::of(planet);
    assert!(band.floor > planet.atmosphere_height, "{band:?}");
    band
}
