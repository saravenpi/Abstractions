//! The shares of a planet's land by biome, counted over a spiral of points.

use bevy::math::DVec3;

use crate::climate::{self, Climate};
use crate::world::globe::MEAN_SEASON;
use crate::world::{Biome, Globe};

/// Points of the sphere the biome shares are counted over.
const POINTS: usize = 240;

/// Shares of the land in desert, grass, forest and snow.
#[derive(Debug, Clone, Copy, Default)]
pub struct Shares {
    /// Share of desert.
    pub desert: f32,
    /// Share of meadow and cherry grove.
    pub grass: f32,
    /// Share of forest and tropical land.
    pub forest: f32,
    /// Share of taiga and snow.
    pub snow: f32,
}

/// Returns the annual mean climate of `globe` at `dir`.
pub fn climate_at(globe: &Globe, dir: DVec3) -> Climate {
    climate::at(globe.setting(), &globe.spot(dir), MEAN_SEASON)
}

/// Returns the shares of the land of `globe` by biome, over a spiral of points.
pub fn shares(globe: &Globe) -> Shares {
    let mut tally = [0u32; 4];
    for i in 0..POINTS {
        let y = 1.0 - 2.0 * (i as f64 + 0.5) / POINTS as f64;
        let a = i as f64 * 2.399_963;
        let r = (1.0 - y * y).sqrt();
        let dir = DVec3::new(r * a.cos(), y, r * a.sin());
        let cold = climate::snow(&climate_at(globe, dir));
        let slot = match globe.biome(dir) {
            _ if cold => 3,
            Biome::Desert => 0,
            Biome::Meadow | Biome::Cherry => 1,
            Biome::Forest | Biome::Tropical => 2,
            Biome::Taiga => 3,
        };
        tally[slot] += 1;
    }
    let share = |n: u32| n as f32 / POINTS as f32;
    Shares {
        desert: share(tally[0]),
        grass: share(tally[1]),
        forest: share(tally[2]),
        snow: share(tally[3]),
    }
}
