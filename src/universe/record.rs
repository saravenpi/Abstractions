//! The planet record: what the physics of a planet says about its air, heat and water.

use std::f64::consts::FRAC_PI_2;

use super::climate::Climate;
use super::hash::Rng;
use super::spec::StarSpec;
use super::units::{EARTH_RADIUS, STANDARD_GRAVITY, SUN_MASS};
use super::{air, class, climate, volatiles, zone};

mod body;

pub(super) use body::Body;

/// The kind of world a planet is, decided by its record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlanetClass {
    /// Liquid water over nearly all of the surface, with scattered small islands.
    Ocean,
    /// Seas and land in balance, like the Earth.
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
/// One record per planet feeds the ground, the sea, the sky, life and the board computer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlanetRecord {
    /// The kind of world, decided by every value below.
    pub class: PlanetClass,
    /// Where the orbit lies against the habitable zone of the star.
    pub zone: Zone,
    /// Starlight reaching the planet, the Earth receiving 1.
    pub flux: f64,
    /// Angle between the spin axis and the axis of the orbit, in radians.
    pub tilt: f64,
    /// Air pressure at sea level, the Earth having 1. Under 0.01 there is no air to speak of.
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
    /// Share of the surface under lakes of liquid ammonia, from 0 to 0.3.
    pub ammonia: f64,
    /// Share of the surface under lakes of liquid methane and ethane, from 0 to 0.3.
    pub hydrocarbon: f64,
    /// Heat the tides of its planet store in a moon, in units of flux: 0 for a planet.
    pub tide: f64,
    /// Carbon the planet has to cycle through its air, sea and rock, from 0 to 1.
    pub carbon: f64,
    /// The gases of the air with their shares, as the planet has them without life. A planet
    /// whose life splits water breathes [`air::Mix::breathed`] of it.
    pub mix: air::Mix,
}

impl PlanetRecord {
    /// Share of the surface the sea covers on a world with seas and land in balance.
    ///
    /// The ground generator is tuned for this share: a planet with it has its sea exactly where
    /// the island generator puts it.
    pub const BALANCED_OCEAN: f64 = 0.6;

    /// The record of a temperate world of seas and islands tilted like the Earth. No planet is
    /// pinned to it: it is the reference a globe built without a planet falls back on.
    pub const TEMPERATE: Self = Self {
        class: PlanetClass::Island,
        zone: Zone::Habitable,
        flux: 1.0,
        tilt: 0.409,
        pressure: 1.0,
        albedo: 0.3,
        greenhouse: 33.0,
        temperature: 288.0,
        water: Water::Liquid,
        ocean: Self::BALANCED_OCEAN,
        ammonia: 0.0,
        hydrocarbon: 0.0,
        tide: 0.0,
        carbon: 0.5,
        mix: air::Mix::HOME,
    };

    /// Returns the share of the surface under each liquid life can use, in the order water,
    /// ammonia, hydrocarbons, molten rock. [`PlanetRecord::ocean`] is the water of a world whose
    /// water is liquid and the molten rock of a lava world.
    pub fn liquids(&self) -> [f64; 4] {
        let sea = |holds: bool| if holds { self.ocean } else { 0.0 };
        let lava = self.class == PlanetClass::Lava;
        [
            sea(!lava && self.water == Water::Liquid),
            self.ammonia,
            self.hydrocarbon,
            sea(lava),
        ]
    }

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
    pub(super) fn derive(star: &StarSpec, body: &Body, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let flux = zone::flux(star.luminosity, body.orbit);
        let (gravity, size) = (body.gravity / STANDARD_GRAVITY, body.radius / EARTH_RADIUS);
        let pressure = air::pressure(rng.unit(), gravity, size, flux);
        let wet = volatiles::water(rng.unit(), flux, star.mass / SUN_MASS);
        let tilt = tilt(rng.unit());
        let carbon = volatiles::carbon(rng.unit(), flux);
        let climate = climate::settle(flux + body.tide, pressure, wet, carbon);
        let lake = |lake: &volatiles::Lake, roll| {
            volatiles::lakes(lake, climate.temperature, pressure, roll)
        };
        let record = Self {
            ocean: class::liquid(pressure, &climate, wet, body.tide),
            ammonia: lake(&volatiles::AMMONIA, rng.unit()),
            hydrocarbon: lake(&volatiles::HYDROCARBON, rng.unit()),
            tide: body.tide,
            carbon,
            ..Self::lit(star, flux, (tilt, pressure), &climate)
        };
        let record = Self {
            class: class::classify(&record),
            ..record
        };
        Self {
            mix: air::born(&record, air::escape(gravity, size), body.tide, seed),
            ..record
        }
    }

    /// Returns the record of a gas giant on an orbit of `orbit` metres, read at its cloud deck.
    ///
    /// `seed` gives the first three draws of [`PlanetRecord::derive`]; the gas stock goes
    /// unused, since a giant holds all of it. The pressure is the cap, the ground nothing: no
    /// liquid covers it and its water is ice, steam or nothing as [`climate::deck`] finds it.
    ///
    /// Ranges: flux 0.000003 to 0.14, pressure 5, albedo 0.34, greenhouse 74 K, temperature 80
    /// to 230 K, ocean 0.
    pub(super) fn giant(star: &StarSpec, orbit: f64, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let flux = zone::flux(star.luminosity, orbit);
        let _stock = rng.unit();
        let wet = rng.unit();
        let climate = climate::deck(flux, wet);
        let record = Self {
            class: PlanetClass::GasGiant,
            ..Self::lit(star, flux, (tilt(rng.unit()), air::PRESSURE_CAP), &climate)
        };
        Self {
            mix: air::born(&record, f64::INFINITY, 0.0, seed),
            ..record
        }
    }

    /// Returns the record of a barren, dry body with its light, its lean, its air and its climate.
    fn lit(star: &StarSpec, flux: f64, (tilt, pressure): (f64, f64), climate: &Climate) -> Self {
        Self {
            class: PlanetClass::Barren,
            zone: zone::zone(star.temperature, flux),
            flux,
            tilt,
            pressure,
            albedo: climate.albedo,
            greenhouse: climate.greenhouse,
            temperature: climate.temperature,
            water: climate.water,
            ocean: 0.0,
            ammonia: 0.0,
            hydrocarbon: 0.0,
            tide: 0.0,
            carbon: 0.0,
            mix: air::Mix::HOME,
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
