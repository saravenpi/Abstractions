//! The air around the listener, as the sound needs it: how dense it is, how fast sound crosses it,
//! and how much of the high end each stretch of it takes.

use super::iso::{Loss, Mixture};
use super::tune::{CORNERS, FIT};
use crate::speed::{sound_at, sound_speed, temperature_over};
use crate::universe::{PlanetRecord, PlanetSpec};
use crate::wind::thickness;

/// How the corner of the low-pass of the air falls with distance: a power law through the corners
/// ISO 9613-1 gives at the two distances of [`FIT`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Absorb {
    /// Corner in hertz at the farther distance of the fit.
    pub far: f32,
    /// Exponent of the distance in the law: negative, the corner falls as the way grows.
    pub slope: f32,
}

impl Absorb {
    /// No absorption: the corner stays far above hearing at any distance.
    pub const NONE: Absorb = Absorb {
        far: CORNERS.1,
        slope: 0.0,
    };

    /// Fits the law to the absorption `loss`.
    pub fn fit(loss: &Loss) -> Self {
        let near = loss.corner(f64::from(FIT.0)) as f32;
        let far = loss.corner(f64::from(FIT.1)) as f32;
        Self {
            far,
            slope: (far / near).ln() / (FIT.1 / FIT.0).ln(),
        }
    }

    /// Returns the corner in hertz of the low-pass that `metres` of this air make.
    pub fn corner(&self, metres: f32) -> f32 {
        let law = self.far * (metres.max(1.0) / FIT.1).powf(self.slope);
        law.clamp(CORNERS.0, CORNERS.1)
    }
}

/// The air one place of a world has.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Air {
    /// Density against the air at sea level at home: 0 in a vacuum.
    pub density: f32,
    /// Speed of sound in metres per second: infinite in a vacuum, where nothing is ever late
    /// because nothing travels.
    pub sound: f32,
    /// How the air takes the high end with distance.
    pub absorb: Absorb,
}

impl Air {
    /// No air at all.
    pub const VACUUM: Air = Air {
        density: 0.0,
        sound: f32::INFINITY,
        absorb: Absorb::NONE,
    };

    /// Returns the air `height` metres over the sea of `planet`: it thins by a factor e over each
    /// scale height and cools as it rises.
    pub fn of(planet: &PlanetSpec, height: f64) -> Self {
        let record = &planet.record;
        let thin = (-height.max(0.0) / planet.scale_height().max(1.0)).exp();
        let state = (record.pressure * thin, temperature_over(planet, height));
        Self::made(
            record,
            state,
            (thickness(record) * thin) as f32,
            sound_at(planet, height),
        )
    }

    /// Returns the air at sea level of a world with the record `record`.
    pub fn sea(record: &PlanetRecord) -> Self {
        let sound = sound_speed(record.temperature, &record.mix);
        let state = (record.pressure, record.temperature);
        Self::made(record, state, thickness(record) as f32, sound)
    }

    /// Returns the air at sea level of a temperate world like home.
    pub fn home() -> Self {
        Self::sea(&PlanetRecord::TEMPERATE)
    }

    /// Returns the air of `record` at a pressure in Earth atmospheres and a temperature in
    /// kelvin, of `density` against home, where sound travels at `sound` metres per second: a
    /// vacuum when there is no density to speak of.
    fn made(record: &PlanetRecord, state: (f64, f64), density: f32, sound: f32) -> Self {
        if density <= 0.0 {
            return Self::VACUUM;
        }
        let gas = Mixture::of(record, state, sound);
        Self {
            density,
            sound,
            absorb: Absorb::fit(&Loss::of(&gas)),
        }
    }
}
