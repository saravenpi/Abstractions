//! Climate: what the planet record means at one place and season, as a pure field.
//!
//! Nothing is stored or simulated: every value is a function of the planet, the place and the
//! season, cheap enough to sample for every vertex and every prop.

use bevy::math::{DVec3, Vec2};

use crate::universe::PlanetSpec;

/// The planet as the climate sees it: its record and the parts of its spec that shape weather.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setting {
    /// Mean surface temperature in kelvin.
    pub temperature: f32,
    /// Air pressure at sea level, the home planet having 1.
    pub pressure: f32,
    /// Share of the surface under the liquid, from 0 to 1.
    pub ocean: f32,
    /// Angle between the spin axis and the axis of the orbit, in radians.
    pub tilt: f32,
    /// Seconds for one turn, negative for a moon that turns against its orbit.
    pub day_seconds: f32,
    /// Surface gravity in metres per second squared.
    pub gravity: f32,
    /// Sea level radius in metres.
    pub radius: f32,
    /// Seed the small variations of the climate are drawn from.
    pub seed: u64,
}

impl Setting {
    /// Returns the setting of a planet or moon.
    pub fn of(spec: &PlanetSpec) -> Self {
        Self {
            temperature: spec.record.temperature as f32,
            pressure: spec.record.pressure as f32,
            ocean: spec.record.ocean as f32,
            tilt: spec.record.tilt as f32,
            day_seconds: spec.day_seconds as f32,
            gravity: spec.gravity as f32,
            radius: spec.radius as f32,
            seed: spec.seed,
        }
    }
}

/// One place on the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Unit direction from the planet centre, in the planet frame: +Y is the spin axis.
    pub dir: DVec3,
    /// Height in metres above the liquid surface, or above the datum on a dry world.
    pub altitude: f32,
    /// How far inland the place is, from about -1 out at sea to about 1 deep inland.
    pub inland: f32,
    /// Rise of the ground towards the east and towards the north, as slopes.
    pub slope: Vec2,
}

/// The climate at one place and season.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Climate {
    /// Air temperature in kelvin.
    pub temperature: f32,
    /// Wetness from 0 (no rain at all) upward; 1 is the wetness of the home planet's coasts.
    pub moisture: f32,
    /// Prevailing wind in metres per second, towards the east and towards the north.
    pub wind: Vec2,
}

/// Returns the climate at `spot` on the planet of `setting` when it is `season` of the way
/// round its orbit, from 0 to 1.
///
/// Until the climate agent fills it, this is the planet mean everywhere.
pub fn at(setting: &Setting, spot: &Spot, season: f32) -> Climate {
    let _ = (spot, season);
    Climate {
        temperature: setting.temperature,
        moisture: setting.ocean,
        wind: Vec2::ZERO,
    }
}
