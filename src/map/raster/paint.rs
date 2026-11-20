//! The colour of one cell of a body picture, from the bare shape of the land: the ground
//! colour of its biome, shaded by its slope and height, or the liquid of the planet itself,
//! darker deeper: water, molten rock or whatever the class floods its low ground with. A planet
//! without a sea has no sea colour.

use bevy::color::Srgba;
use bevy::math::{DVec3, Vec2, Vec3};
use bevy::prelude::Color;

use super::BYTES;
use crate::universe::PlanetClass;
use crate::world::globe::Grain;
use crate::world::{Biome, Globe, Liquid, Surface, ground_rgb};

/// Metres under sea level at which the sea is drawn deep.
const DEPTH: f32 = 40.0;

/// Slope (rise over run) above which land is drawn as rock, as the ground does.
const ROCK_SLOPE: f32 = 1.0;

/// Height above which land is drawn as rock, as the ground does.
const ROCK_HEIGHT: f32 = 38.0;

/// Height above which land is drawn as snow, on the worlds that have it.
const SNOW_HEIGHT: f32 = 56.0;

/// Height under which land is drawn as sand, as the ground does.
const SAND_HEIGHT: f32 = 1.5;

/// Direction the light comes from: the north-west, high in the sky.
const LIGHT: Vec3 = Vec3::new(-0.5, 0.5, 0.7);

/// Height in metres that brightens the land by a fifth.
const HIGH: f32 = 60.0;

/// Returns the pixel of the ground in unit direction `dir`, from about six bare ground samples.
pub fn colour(globe: &Globe, dir: DVec3) -> [u8; BYTES] {
    let (height, slope) = relief(globe, dir);
    let rgb = match sea_colour(globe, height) {
        Some(sea) => sea,
        None => {
            let biome = globe.biomes_seen([dir; 3], Grain::Far)[0];
            let (r, g, b) = ground_rgb(cover(globe, height, slope.length(), biome), biome);
            Vec3::new(r, g, b) * shade(height, slope)
        }
    };
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    [byte(rgb.x), byte(rgb.y), byte(rgb.z), u8::MAX]
}

/// Returns the colour of the liquid of `globe` over a floor at `height` metres, from the shore
/// to the deep, or nothing where that floor is dry: the liquid is the one the world draws, so
/// molten rock is not blue. The colour is in sRGB, from 0 to 1.
pub fn sea_colour(globe: &Globe, height: f32) -> Option<Vec3> {
    if height >= globe.sea_level() {
        return None;
    }
    let liquid = Liquid::of(globe);
    let linear = liquid
        .shallow
        .lerp(liquid.deep, (-height / DEPTH).clamp(0.0, 1.0));
    let shown = Srgba::from(Color::linear_rgb(linear.x, linear.y, linear.z));
    Some(Vec3::new(shown.red, shown.green, shown.blue))
}

/// Returns the bare height and its rise towards the east and the north in direction `dir`.
fn relief(globe: &Globe, dir: DVec3) -> (f32, Vec2) {
    if !globe.has_ground() {
        return (globe.height_seen(dir, Grain::Far), Vec2::ZERO);
    }
    let (height, spot) = globe.glance(dir);
    (height, spot.slope)
}

/// Returns what covers dry land at `height` for a slope and a biome: rock, snow, sand or grass,
/// with no snow on the hot worlds.
fn cover(globe: &Globe, height: f32, slope: f32, biome: Biome) -> Surface {
    let cold = matches!(
        globe.record().class,
        PlanetClass::Ocean | PlanetClass::Island | PlanetClass::Ice
    );
    if slope > ROCK_SLOPE || (height > ROCK_HEIGHT && !(cold && height > SNOW_HEIGHT)) {
        Surface::Rock
    } else if height > SNOW_HEIGHT {
        Surface::Snow
    } else if biome == Biome::Desert || height < SAND_HEIGHT {
        Surface::Sand
    } else {
        Surface::Grass
    }
}

/// Returns how bright land of `height` and `slope` is drawn: lit from the north-west, higher
/// ground a little brighter.
fn shade(height: f32, slope: Vec2) -> f32 {
    let normal = Vec3::new(-slope.x, -slope.y, 1.0).normalize();
    let light = LIGHT.normalize();
    let lit = (normal.dot(light) / light.z).clamp(0.4, 1.4);
    lit * (1.0 + 0.2 * (height / HIGH).clamp(0.0, 1.5))
}
