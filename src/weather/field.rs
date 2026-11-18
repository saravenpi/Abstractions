//! The cloud field: the one truth of where the clouds and the storms of a planet are.
//!
//! The broad cover and the storm are pure functions of the seed, a planet-fixed direction and
//! universe time. The cover map of the shaders, the rain, the thunder, the whiteout and the
//! sound all read them from here, so what is drawn over a place is what happens there.

use bevy::math::{DQuat, DVec3};

use super::grid::{Patch, SNAPSHOT, moment};
use super::noise::broad;
use crate::render::smoothstep as ramp;
use crate::universe::{PlanetClass, PlanetRecord, PlanetSpec};
use crate::wind::thickness;

/// Thickness of the air, against the home planet, from which the sky holds its full weather.
const FULL_AIR: f32 = 0.3;

/// Metres across one cell of the broad noise at the ground of a small planet.
const BROAD: f64 = 1500.0;

/// Fewest and most cells of the broad noise along the radius of a planet. One gives a little
/// over six cells round the smallest planet; ten keeps every cell at least four texels of the
/// cover grid wide on the largest, where the cells grow with the planet.
const CELLS: (f64, f64) = (1.0, 10.0);

/// Metres per second the broad weather drifts east at the equator, round the spin axis.
const DRIFT: f64 = 4.0;

/// Noise level from which a storm starts and the level at which it is full.
const STORM: (f32, f32) = (0.55, 0.8);

/// Seconds of universe time until which the sky of the home planet stays calm, and from which
/// it has its full weather: a new game opens under a mild sky and never in a storm.
const OPENING: (f32, f32) = (300.0, 480.0);

/// Most of the sky that clouds cover while the sky of the home planet is kept calm: too little
/// for rain or snow to fall.
const LIGHT: f32 = 0.35;

/// The noise channel of each value of the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    /// Cloud cover.
    Cloud = 1,
    /// Storm strength.
    Storm = 2,
}

/// The cloud field of one planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Field {
    /// Seed of the planet, which its weather is drawn from.
    pub seed: u64,
    /// Share of a full sky its air can hold: 0 without air, 1 from [`FULL_AIR`] up.
    pub thin: f32,
    /// Whether this is the home planet, whose sky is calm while a new game opens.
    pub home: bool,
    /// Whether the planet has a deck of clouds at all: a gas giant, a class without clouds
    /// and a planet without a cloud band have none, and so no rain or snow either.
    pub deck: bool,
    /// Cells of the broad noise along the radius of the planet.
    pub cells: f64,
    /// Radians per second the broad weather turns east round the spin axis.
    pub turn: f64,
}

/// What the field says over one place at one second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overhead {
    /// Broad cloud cover, from 0 to 1.
    pub cover: f32,
    /// Storm strength, from 0 to 1.
    pub storm: f32,
    /// Share of its weather the sky has: thin air and the calm opening at home lower it.
    pub share: f32,
    /// Share of what falls from the clouds that reaches this height: all of it under the
    /// cloud base, none above the cloud top.
    pub under: f32,
}

impl Field {
    /// Returns the field of a planet of `record`, `seed` and `radius` metres, away from home.
    pub fn new(record: &PlanetRecord, seed: u64, radius: f64) -> Self {
        let thin = (thickness(record) as f32 / FULL_AIR).min(1.0);
        let solid = record.class != PlanetClass::GasGiant;
        Self {
            seed,
            thin,
            home: false,
            deck: solid && thin > 0.0 && record.class.look().clouds > 0.0,
            cells: (radius / BROAD).clamp(CELLS.0, CELLS.1),
            turn: DRIFT / radius.max(1.0),
        }
    }

    /// Returns the field of the planet or moon `spec`, which is the home planet or not.
    pub fn of(spec: &PlanetSpec, home: bool) -> Self {
        let field = Self::new(&spec.record, spec.seed, spec.radius);
        Self {
            home,
            deck: field.deck && spec.cloud_top > spec.cloud_base,
            ..field
        }
    }

    /// Returns the share of its weather the sky has at `seconds`: what the air can hold,
    /// which at home grows from nothing over [`OPENING`].
    pub fn share(&self, seconds: f64) -> f32 {
        self.thin * self.open(seconds)
    }

    /// Returns how far the calm opening of the home planet is over at `seconds`, from 0 to 1.
    fn open(&self, seconds: f64) -> f32 {
        match self.home {
            true => ramp(OPENING.0, OPENING.1, seconds as f32),
            false => 1.0,
        }
    }

    /// Returns the broad noise of `channel` in the planet-fixed unit direction `dir` at
    /// `seconds`, from 0 to 1: noise over the place, turned east with the prevailing wind.
    fn broad(&self, channel: Channel, dir: DVec3, seconds: f64) -> f32 {
        let upwind = DQuat::from_rotation_y(-self.turn * seconds) * dir;
        broad(self.seed, channel as u64, upwind * self.cells, seconds)
    }

    /// Returns the broad cloud cover in direction `dir` at `seconds`, over ground whose
    /// climate is `wet`, from 0 to 1. Wet ground carries more cloud, thin air less, and at
    /// home all but a light cloud waits until [`OPENING`] has passed. It never steps.
    pub fn raw(&self, wet: f32, dir: DVec3, seconds: f64) -> f32 {
        if !self.deck {
            return 0.0;
        }
        let noise = self.broad(Channel::Cloud, dir, seconds);
        let cover = (noise * 1.6 - 0.8 + wet.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        cover.min(1.0 - (1.0 - LIGHT) * (1.0 - self.open(seconds))) * self.thin
    }

    /// Returns the broad cover the sky draws at `seconds` over the place of `patch`, whose
    /// four texels lie over ground of wetness `wet`: [`Field::raw`] at those texels and at the
    /// snapshots before and after, read in between. This is the cover map of the shaders.
    pub fn cover(&self, patch: &Patch, wet: [f32; 4], seconds: f64) -> f32 {
        let (epoch, share) = moment(seconds);
        let dirs = patch.dirs();
        let at = |epoch: i64| {
            let second = epoch as f64 * SNAPSHOT;
            patch.blend(std::array::from_fn(|texel| {
                self.raw(wet[texel], dirs[texel], second)
            }))
        };
        let (before, after) = (at(epoch), at(epoch + 1));
        before + (after - before) * share
    }

    /// Returns the storm strength in direction `dir` at `seconds`, from 0 to 1: storms are
    /// places too, on their own channel of the noise.
    pub fn storm(&self, dir: DVec3, seconds: f64) -> f32 {
        let noise = self.broad(Channel::Storm, dir, seconds);
        ramp(STORM.0, STORM.1, noise) * self.share(seconds)
    }
}
