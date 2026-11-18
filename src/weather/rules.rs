//! The weather rules: what the sky does at one place, from its climate, the seed and the time.
//!
//! Nothing is stored: the same planet, climate and sky overhead always give the same weather.
//! The sky overhead comes from the cloud field, which is a place on the planet and a second.

use bevy::prelude::*;

use super::field::Overhead;
use super::noise::knot;
use crate::climate::Climate;
use crate::render::smoothstep as ramp;
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::wind::thickness;

/// Temperatures in kelvin over which falling water turns from rain to snow.
const FREEZE: (f32, f32) = (276.0, 274.0);

/// Prevailing wind in metres per second that counts as a gale on its own.
const GALE: f32 = 15.0;

/// Wetness of the climate from which bare ground starts to give dust and at which it gives all
/// of it. A desert world is dry everywhere.
const DUSTY: (f32, f32) = (0.35, 0.1);

/// Cloud cover from which a storm starts to show and to thunder, and at which it does in full.
const SQUALL: (f32, f32) = (0.5, 0.9);

/// Broad cloud cover from which rain or snow starts to fall and at which it falls in full.
pub const RAIN: (f32, f32) = (0.55, 1.0);

/// Seconds of one thunder slot: at most one clap in each.
pub const THUNDER_SECONDS: f64 = 9.0;

/// The noise channel of the thunder claps, apart from the channels of the cloud field.
const THUNDER: u64 = 3;

/// What the weather rules need to know about a planet and the ground under the camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet {
    /// Class of the planet: a gas giant has no deck to rain on, a desert lifts dust.
    pub class: PlanetClass,
    /// Thickness of the air against the home planet, 0 without air.
    pub air: f32,
    /// Whether water can fall from its sky: there is some, and the ground is not molten.
    pub falls: bool,
    /// Whether the ground under the camera is bare and can give dust: any ground of a desert or
    /// barren world, and the desert biome of any other. Meadow, forest and snow give none.
    pub bare: bool,
}

impl Planet {
    /// Returns what the weather rules read of the planet of `record`, over the ground its
    /// class has everywhere. Whoever knows the biome under the camera sets `bare`.
    pub fn of(record: &PlanetRecord) -> Self {
        Self {
            class: record.class,
            air: thickness(record) as f32,
            falls: record.water != Water::None && record.class != PlanetClass::Lava,
            bare: matches!(record.class, PlanetClass::Desert | PlanetClass::Barren),
        }
    }

    /// Returns how much dust the ground under the camera can give where the wetness of the
    /// climate is `moisture`: none unless it is bare, all of it on a desert world.
    fn dry(&self, moisture: f32) -> f32 {
        match (self.bare, self.class) {
            (false, _) => 0.0,
            (true, PlanetClass::Desert) => 1.0,
            (true, _) => ramp(DUSTY.0, DUSTY.1, moisture),
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

impl Weather {
    /// Returns how much of a storm shows in the sky: the storm wind under the clouds it needs.
    /// Under a clear sky a storm is only wind, so nothing darkens, thunders or is called a storm.
    pub fn squall(&self) -> f32 {
        self.storm * ramp(SQUALL.0, SQUALL.1, self.cloud)
    }
}

/// Returns the share of the rain and snow of a cloud band from `base` to `top` metres that
/// falls at `altitude`: all of it below the band, easing out on the way up through it.
pub fn under(altitude: f32, base: f32, top: f32) -> f32 {
    match top > base {
        true => 1.0 - ramp(base, top, altitude),
        false => 1.0,
    }
}

/// Returns the share of falling water that freezes into snow at `temperature` kelvin.
pub fn frozen(temperature: f32) -> f32 {
    ramp(FREEZE.0, FREEZE.1, temperature)
}

/// Returns the weather on `planet` where its climate is `climate`, under what the cloud field
/// says `sky` is overhead.
///
/// Without air the sky is still and clear. On a gas giant there is no deck for rain, snow or
/// dust: only storm wind. Rain and snow fall only under thick broad cloud, so a place where it
/// rains is a place the sky draws overcast: rain where the air is wet and above freezing, snow
/// where it is below, and less and less on the way up through the cloud band. Dust rises where
/// the ground is bare and dry and the wind strong: over grass, trees or snow a gale is only wind.
pub fn forecast(planet: &Planet, climate: &Climate, sky: &Overhead) -> Weather {
    if planet.air <= 0.0 {
        return Weather::default();
    }
    if planet.class == PlanetClass::GasGiant {
        return Weather {
            storm: sky.storm,
            wind: climate.wind,
            ..default()
        };
    }
    let wet = ramp(0.25, 0.6, climate.moisture.clamp(0.0, 1.0));
    let falls = f32::from(u8::from(planet.falls));
    let falling = falls * ramp(RAIN.0, RAIN.1, sky.cover) * wet * sky.under;
    let snow = frozen(climate.temperature);
    let gale = (sky.storm + 0.5 * (climate.wind.length() / GALE).min(1.0)).min(1.0);
    Weather {
        cloud: sky.cover,
        rain: falling * (1.0 - snow),
        snow: falling * snow,
        dust: planet.dry(climate.moisture) * ramp(0.4, 0.8, gale) * sky.share,
        storm: sky.storm,
        wind: climate.wind,
    }
}

/// Returns the level of the far thunder clap in slot `slot` of [`THUNDER_SECONDS`] on the
/// planet of `seed` under `weather`, or nothing. Claps come only out of storm clouds, and more
/// often the stronger the storm.
pub fn thunder(seed: u64, slot: i64, weather: &Weather) -> Option<f32> {
    let charge = weather.squall();
    let roll = knot(seed, THUNDER, slot);
    (roll < 0.6 * charge).then_some(0.4 + 0.6 * charge)
}
