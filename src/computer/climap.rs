//! The climate map of the card: the whole planet spread flat, longitude across and latitude
//! down, each cell the grey of the biome its climate gives at sea level.
//!
//! The sea is black, snow is white and the six biomes are six greys between, so the map stays
//! black and white like the rest of the interface. It is pure: the same planet and season
//! always give the same pixels.

use std::f64::consts::{PI, TAU};

use bevy::math::Vec2;

use super::season::direction;
use crate::climate::{self, Climate, Setting, Spot};
use crate::universe::{PlanetClass, PlanetSpec};
use crate::world::{Biome, Globe};

/// Cells across the map, one per 7.5 degrees of longitude.
pub const COLS: usize = 48;

/// Cells down the map, one per 7.5 degrees of latitude.
pub const ROWS: usize = 24;

/// Bytes of one pixel: red, green, blue and alpha.
pub const BYTES: usize = 4;

/// Air temperature in kelvin under which the ground is drawn as snow.
pub const SNOW: f32 = 263.15;

/// Grey of the sea.
pub const SEA: u8 = 0;

/// Grey of snow.
pub const WHITE: u8 = 255;

/// Returns the grey a biome is drawn in: the denser its plants, the darker.
pub fn tone(biome: Biome) -> u8 {
    match biome {
        Biome::Tropical => 64,
        Biome::Forest => 96,
        Biome::Cherry => 128,
        Biome::Meadow => 160,
        Biome::Taiga => 192,
        Biome::Desert => 224,
    }
}

/// Returns the grey of one cell: black under the sea, white under snow, else its biome.
pub fn grey(climate: &Climate, sea: bool) -> u8 {
    match (sea, climate.temperature < SNOW) {
        (true, _) => SEA,
        (false, true) => WHITE,
        (false, false) => tone(crate::climate::biome(climate, 0.0)),
    }
}

/// Returns true when the planet has ground to map: every class but a gas giant.
pub fn mapped(spec: &PlanetSpec) -> bool {
    spec.record.class != PlanetClass::GasGiant
}

/// Returns the pixels of the climate map of `spec` when it is `season` of the way round its
/// year, row by row from the north, each pixel a grey.
pub fn grid(spec: &PlanetSpec, season: f32) -> Vec<u8> {
    let (globe, setting) = (Globe::of(spec), Setting::of(spec));
    let mut pixels = Vec::with_capacity(COLS * ROWS * BYTES);
    for cell in 0..COLS * ROWS {
        let lon = ((cell % COLS) as f64 + 0.5) / COLS as f64 * TAU - PI;
        let lat = 0.5 * PI - ((cell / COLS) as f64 + 0.5) / ROWS as f64 * PI;
        let dir = direction(lon, lat);
        let spot = Spot {
            dir,
            altitude: 0.0,
            inland: 0.0,
            slope: Vec2::ZERO,
        };
        let sea = globe.height(dir) < globe.sea_level();
        let shade = grey(&climate::at(&setting, &spot, season), sea);
        pixels.extend([shade, shade, shade, u8::MAX]);
    }
    pixels
}
