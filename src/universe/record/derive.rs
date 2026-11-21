//! Deriving the record of a solid body from its star, its orbit and its draws, as drawn or with
//! its air and its carbon nudged, the one way the home of a new world is fitted to the Earth.

use super::{Body, PlanetRecord, tilt};
use crate::universe::climate::{self, Climate};
use crate::universe::hash::Rng;
use crate::universe::spec::StarSpec;
use crate::universe::units::{EARTH_RADIUS, STANDARD_GRAVITY, SUN_MASS};
use crate::universe::{air, class, volatiles, zone};

/// What a record is given in place of its own draws: nothing for a planet as it was born.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(in crate::universe) struct Nudge {
    /// Air pressure at sea level, the Earth having 1, in place of the drawn one.
    pub pressure: Option<f64>,
    /// Carbon to cycle, from 0 to 1, in place of the drawn supply.
    pub carbon: Option<f64>,
}

/// The draws of a record that the climate settles from: starlight, tilt, pressure, water and
/// carbon.
type Drawn = (f64, f64, f64, f64, f64);

impl PlanetRecord {
    /// Returns the record of `body` around `star`, with its draws from `seed`.
    ///
    /// The seed gives six draws: the gas the body was born with, its water, its tilt, its
    /// carbon and its lakes of ammonia and of hydrocarbons. Starlight, air, climate and class
    /// then follow in that order, each from the ones before, so the same body always gets the
    /// same record. The heat of the tides adds to the starlight the ground absorbs, so it warms
    /// the climate, but neither the flux nor the zone, which are the starlight alone.
    ///
    /// Ranges: flux 0.000003 to 260,000, tilt 0 to a quarter turn, pressure 0 to 5, albedo 0.08
    /// to 0.6, greenhouse 0 to 1,500 K, temperature 5 to 7,000 K, ocean 0 to 1.
    pub(in crate::universe) fn derive(star: &StarSpec, body: &Body, seed: u64) -> Self {
        Self::nudged(star, body, seed, Nudge::default())
    }

    /// Returns the record [`PlanetRecord::derive`] gives, with the pressure and the carbon of
    /// `nudge` in place of the drawn ones. Every draw is still made, so the others never shift,
    /// and the climate, the seas, the lakes, the class and the air follow from the nudged values.
    pub(in crate::universe) fn nudged(
        star: &StarSpec,
        body: &Body,
        seed: u64,
        nudge: Nudge,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let flux = zone::flux(star.luminosity, body.orbit);
        let (gravity, size) = (body.gravity / STANDARD_GRAVITY, body.radius / EARTH_RADIUS);
        let pressure = air::pressure(rng.unit(), gravity, size, flux);
        let wet = volatiles::water(rng.unit(), flux, star.mass / SUN_MASS);
        let tilt = tilt(rng.unit());
        let carbon = volatiles::carbon(rng.unit(), flux);
        let pressure = nudge.pressure.unwrap_or(pressure);
        let carbon = nudge.carbon.unwrap_or(carbon);
        let record = Self::settled(star, body, (flux, tilt, pressure, wet, carbon), &mut rng);
        let record = Self {
            class: class::classify(&record),
            ..record
        };
        Self {
            mix: air::born(&record, air::escape(gravity, size), body.tide, seed),
            ..record
        }
    }

    /// Returns the record of `body` around `star` once its climate settled from its draws
    /// `drawn`, with its seas and its lakes, the lakes drawn from `rng`. Its class and its air
    /// are still those of a barren world.
    fn settled(star: &StarSpec, body: &Body, drawn: Drawn, rng: &mut Rng) -> Self {
        let (flux, tilt, pressure, wet, carbon) = drawn;
        let outer = zone::outer(star.temperature);
        let climate: Climate = climate::settle(flux + body.tide, outer, pressure, wet, carbon);
        let lake = |lake: &volatiles::Lake, roll| {
            volatiles::lakes(lake, climate.temperature, pressure, roll)
        };
        Self {
            ocean: class::liquid(pressure, &climate, wet, body.tide),
            ammonia: lake(&volatiles::AMMONIA, rng.unit()),
            hydrocarbon: lake(&volatiles::HYDROCARBON, rng.unit()),
            tide: body.tide,
            carbon,
            ..Self::lit(star, flux, (tilt, pressure), &climate)
        }
    }
}
