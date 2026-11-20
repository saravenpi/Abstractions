//! The planet as the body rules see it.

use crate::universe::{PlanetSpec, StarSpec};

/// What a planet imposes on the bodies that live on it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    /// Surface gravity in metres per second squared.
    pub gravity: f32,
    /// Air pressure at sea level, the home planet having 1.
    pub pressure: f32,
    /// Mean surface temperature in kelvin.
    pub temperature: f32,
    /// Starlight reaching the planet, the home planet receiving 1.
    pub flux: f32,
    /// Sea level radius in metres.
    pub radius: f32,
    /// Share of the surface under the liquid, 0 to 1.
    pub ocean: f32,
    /// Surface temperature of the star in kelvin.
    pub star: f32,
    /// Angle between the spin axis and the axis of the orbit, in radians: how hard the seasons swing.
    pub tilt: f32,
    /// Share of the surface under each liquid, in the order of `Solvent`: water, ammonia,
    /// hydrocarbons and molten rock. Zero where the planet holds none of it.
    pub liquids: [f32; 4],
}

impl World {
    /// Returns the world of `planet` around `star`.
    pub fn of(planet: &PlanetSpec, star: &StarSpec) -> Self {
        let record = &planet.record;
        Self {
            gravity: planet.gravity as f32,
            pressure: record.pressure as f32,
            temperature: record.temperature as f32,
            flux: record.flux as f32,
            radius: planet.radius as f32,
            ocean: record.ocean as f32,
            star: star.temperature as f32,
            tilt: record.tilt as f32,
            liquids: record.liquids().map(|share| share as f32),
        }
    }
}
