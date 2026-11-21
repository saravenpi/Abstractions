//! What the ground of a site must be beyond its relief: firm cover, and not deep in a harsh biome.

use std::f32::consts::TAU;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{FOOTPRINT, HARSH, HEART, STARTS};
use crate::climate;
use crate::universe::MILD;
use crate::world::globe::MEAN_SEASON;
use crate::world::{Biome, Globe, Surface, Terrain};

/// Points on each circle the cover and the biome are checked at.
const SAMPLES: u32 = 8;

/// Returns `SAMPLES` points evenly spaced on the circle of `radius` around `at`.
fn ring(at: Vec2, radius: f32) -> impl Iterator<Item = Vec2> {
    (0..SAMPLES).map(move |i| at + Vec2::from_angle(i as f32 / SAMPLES as f32 * TAU) * radius)
}

/// Returns true when the footprint around `at` in the metric `view` is firm ground: grass, sand,
/// rock or snow at its centre and on two circles, never a pond or a river bed.
pub fn firm(view: &Terrain, at: Vec2) -> bool {
    let radius = FOOTPRINT as f32;
    let mut points = std::iter::once(at)
        .chain(ring(at, radius * 0.5))
        .chain(ring(at, radius));
    points.all(|p| {
        matches!(
            view.surface(p.x, p.y),
            Surface::Grass | Surface::Sand | Surface::Rock | Surface::Snow
        )
    })
}

/// Returns true when `at` in the metric `view` lies in the heart of a taiga or a desert: its biome
/// is one of them, and so is every point of a circle of [`HEART`] metres around it. The edge of
/// one, near meadow or forest, still takes the base.
pub fn harsh(view: &Terrain, at: Vec2) -> bool {
    let biome = view.biome(at.x, at.y);
    matches!(biome, Biome::Taiga | Biome::Desert)
        && ring(at, HEART).all(|p| view.biome(p.x, p.y) == biome)
}

/// Returns what a site at `at` in the metric `view` costs in metres for the harsh ground around it:
/// [`HARSH`] metres when its centre or a point of the circle of the first walk around it is taiga
/// or desert, so that meadow and forest all around win when they are near.
pub fn fringe(view: &Terrain, at: Vec2) -> f32 {
    let (radius, cost) = HARSH;
    let mut points = std::iter::once(at).chain(ring(at, radius));
    let rough = points.any(|p| matches!(view.biome(p.x, p.y), Biome::Taiga | Biome::Desert));
    cost * f32::from(u8::from(rough))
}

/// Returns true when life thrives around unit direction `dir` of `globe`: its annual mean air is
/// between 0 and 35 C and its biome is not a desert, so that plants stand there.
fn thrives(globe: &Globe, dir: DVec3) -> bool {
    let air = climate::at(globe.setting(), &globe.spot(dir), MEAN_SEASON);
    let (cold, hot) = (MILD.0 as f32, MILD.1 as f32);
    (cold..=hot).contains(&air.temperature) && globe.biome(dir) != Biome::Desert
}

/// Returns the planet directions around which the base is looked for, in order: the shores of the land at a
/// mild latitude that the drainage grid of `globe` finds, river mouths first, those where life thrives
/// before the others, see [`thrives`]. A planet whose grid finds none, a dry one, is looked at on a ring
/// instead, see [`spread`]. The climate is read lazily, shore by shore, only as far as the search goes.
pub fn starts(globe: &Globe) -> impl Iterator<Item = DVec3> + '_ {
    let shores = globe.landfalls();
    let shores = match shores.is_empty() {
        true => spread(globe.seed()).collect(),
        false => shores,
    };
    let alive = shores
        .clone()
        .into_iter()
        .filter(|dir| thrives(globe, *dir));
    alive.chain(shores.into_iter().filter(|dir| !thrives(globe, *dir)))
}

/// Returns the planet directions of the ring of fallback starts, in order: on a mild northern latitude,
/// the first at a longitude `seed` picks and the others spread east of it.
pub fn spread(seed: u64) -> impl Iterator<Item = DVec3> + Clone {
    let (latitude, count) = STARTS;
    let turn = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 11) as f64 / (1u64 << 53) as f64;
    let lat = f64::from(latitude).to_radians();
    (0..count).map(move |i| {
        let east = (turn + f64::from(i) / f64::from(count)) * std::f64::consts::TAU;
        DVec3::new(lat.cos() * east.sin(), lat.sin(), lat.cos() * east.cos())
    })
}
