//! The colour of one cell of a body picture, from the bare shape of the land: the colour the meshes of the
//! world draw there, through the one rule of the lands, or the liquid of the planet itself, darker deeper:
//! water, molten rock or whatever the class floods its low ground with. A planet without a sea has no sea
//! colour.

use bevy::color::Srgba;
use bevy::math::{DVec3, Vec2, Vec3};
use bevy::prelude::Color;

use super::BYTES;
use crate::world::globe::Grain;
use crate::world::{Globe, Liquid, Reading};

/// Metres under sea level at which the sea is drawn deep.
const DEPTH: f32 = 40.0;

/// Returns the pixel of the ground in unit direction `dir`, from about six bare ground samples.
pub fn colour(globe: &Globe, dir: DVec3) -> [u8; BYTES] {
    let (height, slope) = relief(globe, dir);
    let rgb = match sea_colour(globe, height) {
        Some(sea) => sea,
        None => {
            let biome = globe.biomes_seen([dir; 3], Grain::Far)[0];
            globe.land_colour(
                Reading {
                    height,
                    slope: slope.length(),
                },
                biome,
            )
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
