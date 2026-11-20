//! The ambient light of a place by direction. A surface that faces the sky takes the colour of
//! the sky, one that faces the ground takes the light the ground sends back, and at dusk the
//! side that faces the set star takes the glow of the twilight arch. `relit` in `daylight.wesl`
//! applies the same rules on the GPU: [`hemisphere`], [`bounce`] and [`twilight`] mirror its
//! `hemisphere`, `bounce_of` and `twilight`, and the constants carry the same names.
//!
//! The colour of the ground goes to the shader in the free fourth slots of the place block, in
//! `air.w`, `day.w` and `eye_sky.w`, which [`with_ground`] fills. The rules themselves run on the
//! GPU, and their twins here are what the tests check.

use bevy::prelude::*;

use super::PlaceUniform;
#[cfg(test)]
use crate::render::celestial::smoothstep;
#[cfg(test)]
use crate::render::scatter::luminance;

/// Most light the ground sends back, as a share of the ambient light of the sky. A real ground
/// sends back more than the sky gives here, because the game keeps its sky dim against its sun;
/// this keeps the fill under a surface from outshining the sky over it. A share of 1.
#[cfg(test)]
pub const BOUNCE_CAP: f32 = 1.5;

/// Share of the ambient level the twilight arch adds on the side of the set star, at its
/// brightest. A share of 1.
#[cfg(test)]
pub const TWILIGHT_SHARE: f32 = 0.35;

/// Sines of the height of the star at which the twilight arch comes up, peaks and is gone.
#[cfg(test)]
pub const TWILIGHT_EDGES: Vec3 = Vec3::new(-0.18, -0.02, 0.12);

/// Returns the ambient light on a surface whose normal leans `tilt` toward the sky, from -1 for
/// a normal that points at the ground to 1 for one that points at the sky: all `sky` up, all
/// `bounce` down, and half of each across.
#[cfg(test)]
pub fn hemisphere(sky: Vec3, bounce: Vec3, tilt: f32) -> Vec3 {
    bounce.lerp(sky, 0.5 + 0.5 * tilt.clamp(-1.0, 1.0))
}

/// Returns the ambient light the ground of linear colour `ground` sends up to a surface that
/// faces it, under `sun` as the shader works it out: the star on level ground at the sine
/// `height`, spread over the half sphere, plus the sky, and no more than [`BOUNCE_CAP`] times the sky.
#[cfg(test)]
pub fn bounce(ground: Vec3, sun: Vec3, height: f32, sky: Vec3) -> Vec3 {
    let level = luminance(sun) * height.max(0.0) / std::f32::consts::PI + luminance(sky);
    let given = ground * level;
    let (top, seen) = (BOUNCE_CAP * luminance(sky), luminance(given));
    match seen > top {
        true => given * (top / seen.max(1.0e-6)),
        false => given,
    }
}

/// Returns how strong the twilight arch is while the star stands at the sine `height`: nothing
/// in full day and in deep night, and most as the star sets.
#[cfg(test)]
pub fn twilight(height: f32) -> f32 {
    smoothstep(TWILIGHT_EDGES.x, TWILIGHT_EDGES.y, height)
        * (1.0 - smoothstep(TWILIGHT_EDGES.y, TWILIGHT_EDGES.z, height))
}

/// Returns `place` carrying the linear colour `ground` of the ground in its free slots.
pub fn with_ground(mut place: PlaceUniform, ground: Vec3) -> PlaceUniform {
    place.air.w = ground.x;
    place.day.w = ground.y;
    place.eye_sky.w = ground.z;
    place
}

/// Returns the linear colour of the ground a place block carries.
#[cfg(test)]
pub fn ground(place: &PlaceUniform) -> Vec3 {
    Vec3::new(place.air.w, place.day.w, place.eye_sky.w)
}
