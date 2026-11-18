//! The weather rules: what the sky does at one place, from its climate, the seed and the time.
//!
//! Nothing is stored: the same planet, climate and second always give the same weather. It
//! changes over minutes because it follows slow seeded noise in time.

use bevy::prelude::*;

use super::noise::{knot, slow};
use crate::climate::Climate;
use crate::render::smoothstep as ramp;
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::wind::thickness;

/// Thickness of the air, against the home planet, from which the sky holds its full weather.
const FULL_AIR: f32 = 0.3;

/// Temperatures in kelvin over which falling water turns from rain to snow.
const FREEZE: (f32, f32) = (276.0, 274.0);

/// Noise level from which a storm starts and the level at which it is full.
const STORM: (f32, f32) = (0.55, 0.8);

/// Prevailing wind in metres per second that counts as a gale on its own.
const GALE: f32 = 15.0;

/// Seconds of one thunder slot: at most one clap in each.
pub const THUNDER_SECONDS: f64 = 9.0;

/// The noise channel of each weather value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Channel {
    /// Cloud cover.
    Cloud = 1,
    /// Storm strength.
    Storm = 2,
    /// Thunder claps.
    Thunder = 3,
}

/// What the weather rules need to know about a planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet {
    /// Seed of the planet, which its weather is drawn from.
    pub seed: u64,
    /// Class of the planet: a gas giant has no deck to rain on, a desert lifts dust.
    pub class: PlanetClass,
    /// Thickness of the air against the home planet, 0 without air.
    pub air: f32,
    /// Whether water can fall from its sky: there is some, and the ground is not molten.
    pub falls: bool,
}

impl Planet {
    /// Returns what the weather rules read of the planet of `record` and `seed`.
    pub fn of(record: &PlanetRecord, seed: u64) -> Self {
        Self {
            seed,
            class: record.class,
            air: thickness(record) as f32,
            falls: record.water != Water::None && record.class != PlanetClass::Lava,
        }
    }
}

/// The weather at the camera, each value from 0 to 1.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Weather {
    /// Share of the sky under clouds.
    pub cloud: f32,
    /// How hard it rains.
    pub rain: f32,
    /// How hard it snows.
    pub snow: f32,
    /// How much dust the wind lifts.
    pub dust: f32,
    /// How stormy the wind is.
    pub storm: f32,
    /// Prevailing wind of the climate in metres per second, towards the east and the north.
    pub wind: Vec2,
}

/// Returns the share of falling water that freezes into snow at `temperature` kelvin.
pub fn frozen(temperature: f32) -> f32 {
    ramp(FREEZE.0, FREEZE.1, temperature)
}

/// Returns the weather on `planet` where its climate is `climate`, at `seconds` of universe time.
///
/// Without air the sky is still and clear. On a gas giant there is no deck for rain, snow or
/// dust: only storm wind. Rain falls where the air is wet and above freezing, snow where it is
/// below, and dust rises where the ground is dry and the wind strong.
pub fn forecast(planet: &Planet, climate: &Climate, seconds: f64) -> Weather {
    if planet.air <= 0.0 {
        return Weather::default();
    }
    let thin = (planet.air / FULL_AIR).min(1.0);
    let noise = |channel: Channel| slow(planet.seed, channel as u64, seconds);
    let storm = ramp(STORM.0, STORM.1, noise(Channel::Storm)) * thin;
    if planet.class == PlanetClass::GasGiant {
        return Weather {
            storm,
            wind: climate.wind,
            ..default()
        };
    }
    let wet = climate.moisture.clamp(0.0, 1.0);
    let cloud = ((noise(Channel::Cloud) * 1.6 - 0.8 + wet).clamp(0.0, 1.0)) * thin;
    let falling = f32::from(u8::from(planet.falls)) * ramp(0.55, 1.0, cloud) * ramp(0.25, 0.6, wet);
    let snow = frozen(climate.temperature);
    let dry = match planet.class {
        PlanetClass::Desert => 1.0,
        _ => ramp(0.35, 0.1, climate.moisture),
    };
    let gale = (storm + 0.5 * (climate.wind.length() / GALE).min(1.0)).min(1.0);
    Weather {
        cloud,
        rain: falling * (1.0 - snow),
        snow: falling * snow,
        dust: dry * ramp(0.4, 0.8, gale) * thin,
        storm,
        wind: climate.wind,
    }
}

/// Returns the level of the far thunder clap in slot `slot` of [`THUNDER_SECONDS`] on the
/// planet of `seed` under `weather`, or nothing. Claps come only out of storm clouds, and more
/// often the stronger the storm.
pub fn thunder(seed: u64, slot: i64, weather: &Weather) -> Option<f32> {
    let charge = weather.storm * ramp(0.5, 0.9, weather.cloud);
    let roll = knot(seed, Channel::Thunder as u64, slot);
    (roll < 0.6 * charge).then_some(0.4 + 0.6 * charge)
}
