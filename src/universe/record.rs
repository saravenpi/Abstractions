//! The planet record: what the physics of a planet says about its air, heat and water.

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
}

impl PlanetClass {
    /// Every class, in a stable order.
    pub const ALL: [PlanetClass; 6] = [
        PlanetClass::Ocean,
        PlanetClass::Island,
        PlanetClass::Desert,
        PlanetClass::Ice,
        PlanetClass::Lava,
        PlanetClass::Barren,
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
}
