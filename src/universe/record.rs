//! The planet record: what the physics of a planet says about its air, heat and water.

use std::f64::consts::FRAC_PI_2;

use super::hash::Rng;
use super::spec::StarSpec;
use super::{air, class, climate, zone};

/// The kind of world a planet is, decided by its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetClass {
    /// Liquid water over nearly all of the surface, with scattered small islands.
    Ocean,
    /// Seas and land in balance, like the home planet.
    Island,
    /// Air but no open water: sand, rock and dry basins.
    Desert,
    /// Water frozen solid over the whole surface.
    Ice,
    /// Hot enough for molten rock to pool in the low ground.
    Lava,
    /// No air to speak of: bare rock under a black sky.
    Barren,
    /// A massive world of deep air with no ground to stand on, only cloud decks to skim.
    GasGiant,
}

impl PlanetClass {
    /// Every class, in a stable order.
    pub const ALL: [PlanetClass; 7] = [
        PlanetClass::Ocean,
        PlanetClass::Island,
        PlanetClass::Desert,
        PlanetClass::Ice,
        PlanetClass::Lava,
        PlanetClass::Barren,
        PlanetClass::GasGiant,
    ];
}

/// The state water is in at the surface of a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Water {
    /// No water at all.
    None,
    /// Frozen.
    Ice,
    /// Liquid.
    Liquid,
    /// Boiled into the air.
    Steam,
}

/// Where an orbit lies against the habitable zone of its star.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    /// Closer than the inner edge: water boils away.
    Hot,
    /// Between both edges: a planet with enough air can keep oceans.
    Habitable,
    /// Beyond the outer edge: water freezes.
    Cold,
}

/// The physical record of one planet, derived from its star, its orbit and its seed.
///
/// One record per planet feeds the ground, the sea, the sky and the board computer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetRecord {
    /// The kind of world, decided by every value below.
    pub class: PlanetClass,
    /// Where the orbit lies against the habitable zone of the star.
    pub zone: Zone,
    /// Starlight reaching the planet, the home planet receiving 1.
    pub flux: f64,
    /// Angle between the spin axis and the axis of the orbit, in radians.
    pub tilt: f64,
    /// Air pressure at sea level, the home planet having 1. Under 0.01 there is no air to speak of.
    pub pressure: f64,
    /// Share of the starlight sent back to space, from 0 to 1.
    pub albedo: f64,
    /// Warming by the air in kelvin.
    pub greenhouse: f64,
    /// Mean surface temperature in kelvin.
    pub temperature: f64,
    /// The state of water at the surface.
    pub water: Water,
    /// Share of the surface covered by the liquid of the planet, water or molten rock, from 0 to 1.
    pub ocean: f64,
}

impl PlanetRecord {
    /// Share of the surface the sea covers on a world with seas and land in balance.
    ///
    /// The ground generator is tuned for this share: a planet with it has its sea exactly where
    /// the island generator puts it.
    pub const BALANCED_OCEAN: f64 = 0.6;

    /// The record of the home planet: a temperate world of seas and islands.
    pub const TEMPERATE: Self = Self {
        class: PlanetClass::Island,
        zone: Zone::Habitable,
        flux: 1.0,
        tilt: 0.0,
        pressure: 1.0,
        albedo: 0.3,
        greenhouse: 33.0,
        temperature: 288.0,
        water: Water::Liquid,
        ocean: Self::BALANCED_OCEAN,
    };

    /// Returns the record of a planet on an orbit of `orbit` metres around `star`.
    ///
    /// `gravity` and `size` are the surface gravity and the radius over those of the home
    /// planet. `seed` gives three draws: the gas the planet was born with, its water and its
    /// tilt. Starlight, air, climate and class then follow in that order, each from the ones
    /// before, so the same planet always gets the same record.
    ///
    /// Ranges: flux 0.005 to 58,000, tilt 0 to a quarter turn, pressure 0 to 5, albedo 0.08 to
    /// 0.6, greenhouse 0 to 1,500 K, temperature 40 to 6,000 K, ocean 0 to 1.
    pub(super) fn derive(star: &StarSpec, orbit: f64, gravity: f64, size: f64, seed: u64) -> Self {
        Self::heated(star, orbit, [gravity, size], seed, 0.0)
    }

    /// Returns the record of a body lit like a planet on an orbit of `orbit` metres, which also
    /// warms from inside by `tide`, a heat flow in units of the starlight the home planet gets.
    ///
    /// `body` holds the gravity and the radius over those of the home planet, `seed` gives the
    /// draws of [`PlanetRecord::derive`]. The heat adds to the starlight the ground absorbs, so
    /// it warms the climate, but neither the flux nor the zone, which are the starlight alone,
    /// nor the air, which the star blows away and the inner heat does not. A tide of 0 is the
    /// record of a planet, bit for bit.
    pub(super) fn heated(
        star: &StarSpec,
        orbit: f64,
        body: [f64; 2],
        seed: u64,
        tide: f64,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let flux = zone::flux(star.luminosity, orbit);
        let pressure = air::pressure(rng.unit(), body[0], body[1], flux);
        let wet = rng.unit();
        let climate = climate::settle(flux + tide, pressure, wet);
        let record = Self {
            class: PlanetClass::Barren,
            zone: zone::zone(star.temperature, flux),
            flux,
            tilt: tilt(rng.unit()),
            pressure,
            albedo: climate.albedo,
            greenhouse: climate.greenhouse,
            temperature: climate.temperature,
            water: climate.water,
            ocean: class::liquid(pressure, &climate, wet),
        };
        Self {
            class: class::classify(&record),
            ..record
        }
    }

    /// Returns the record of a gas giant on an orbit of `orbit` metres, read at its cloud deck.
    ///
    /// `seed` gives the same three draws as [`PlanetRecord::derive`]; the gas stock goes unused,
    /// since a giant holds all of it. The pressure is the cap, the ground nothing: no liquid
    /// covers it and its water is ice, steam or nothing as [`climate::deck`] finds it.
    ///
    /// Ranges: flux 0.005 to 0.14 (up to 0.26 for the giant of the home system), pressure 5,
    /// albedo 0.34, greenhouse 74 K, temperature 140 to 260 K, ocean 0.
    pub(super) fn giant(star: &StarSpec, orbit: f64, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let flux = zone::flux(star.luminosity, orbit);
        let _stock = rng.unit();
        let wet = rng.unit();
        let climate = climate::deck(flux, wet);
        Self {
            class: PlanetClass::GasGiant,
            zone: zone::zone(star.temperature, flux),
            flux,
            tilt: tilt(rng.unit()),
            pressure: air::PRESSURE_CAP,
            albedo: climate.albedo,
            greenhouse: climate.greenhouse,
            temperature: climate.temperature,
            water: climate.water,
            ocean: 0.0,
        }
    }
}

/// Returns the tilt in radians of a spin axis for a draw `roll` from 0 to 1.
///
/// The tilt is a quarter turn times the cube of the draw: 7 planets in 10 lean by less than
/// 30 degrees and 1 in 25 lies on its side, past 80 degrees.
fn tilt(roll: f64) -> f64 {
    FRAC_PI_2 * roll.powi(3)
}
