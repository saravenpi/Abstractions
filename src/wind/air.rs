//! How thick the air of a planet is, for everything the air carries: drag, lift, wind and sound.

use bevy::prelude::*;

pub use crate::universe::NO_AIR;
use crate::universe::PlanetRecord;

/// Thickest air the flight and the sound follow, against the home planet. Above it the air
/// only gets heavier on paper, so that no planet turns flying into wading.
pub const THICKEST: f64 = 3.0;

/// Returns how thick the air of a planet is against the home planet: its pressure, held under
/// [`THICKEST`], and exactly zero under [`NO_AIR`] or when the pressure is not a number.
///
/// Drag, lift, wind and the sounds of the air all scale with it, so that a planet with the
/// home pressure gives 1 and changes nothing.
pub fn thickness(record: &PlanetRecord) -> f64 {
    if record.pressure >= NO_AIR {
        record.pressure.min(THICKEST)
    } else {
        0.0
    }
}

/// Returns what the wind strength is multiplied by in air of `thickness`: nothing without air,
/// 1 at home, and the square root in between and above, since thin air moves fast but pushes
/// little and thick air pushes hard but moves slowly.
pub fn breeze(thickness: f32) -> f32 {
    thickness.max(0.0).sqrt()
}

/// How thick the air of the planet under the render anchor is against the home planet: 0
/// without air, 1 at home and away from any planet.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Thickness(pub f32);

impl Default for Thickness {
    /// The air of the home planet.
    fn default() -> Self {
        Self(1.0)
    }
}

/// The pressures every air rule is tested at: none, under the limit, thin, home and the thickest.
#[cfg(test)]
pub const PRESSURES: [f64; 5] = [0.0, 0.005, 0.3, 1.0, 3.0];

/// Returns one hand-built record per planet class, in the order of `PlanetClass::ALL`, for the
/// tests of everything that follows the air and the liquid of a planet.
#[cfg(test)]
pub fn samples() -> [PlanetRecord; 6] {
    use crate::universe::{PlanetClass, Water};
    let world = |class, pressure, water, ocean| PlanetRecord {
        class,
        pressure,
        water,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    [
        world(PlanetClass::Ocean, 1.4, Water::Liquid, 0.92),
        PlanetRecord::TEMPERATE,
        world(PlanetClass::Desert, 0.4, Water::None, 0.0),
        world(PlanetClass::Ice, 0.7, Water::Ice, 0.5),
        world(PlanetClass::Lava, 2.5, Water::Steam, 0.3),
        world(PlanetClass::Barren, 0.0, Water::None, 0.0),
    ]
}
