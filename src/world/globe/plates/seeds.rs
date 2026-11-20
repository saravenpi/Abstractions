//! The seeds of the plates: where each one is centred, what crust it carries and how it turns.

use std::f64::consts::{PI, TAU};

use bevy::math::{DQuat, DVec3, EulerRot};

use super::Crust;
use super::interior::Interior;
use super::tune::{COUNT, JITTER, SPACING};
use crate::world::globe::roll::{roll, rolls};

/// The grid cell the rolls of the plates are drawn at.
const KEY: [i64; 3] = [0x91A7E, 0, 0];

/// Returns how many plates planet `seed` has: more where the crust moves faster, and a seeded share.
pub fn count(seed: u64, interior: &Interior) -> usize {
    let share = 0.4 * roll(seed, KEY, 0) + 0.6 * interior.mobility;
    COUNT.0 + ((COUNT.1 - COUNT.0) as f32 * share).round() as usize
}

/// Returns the chord between two neighbour seeds among `count`, on the unit sphere.
pub fn spacing(count: usize) -> f64 {
    SPACING * (4.0 * PI / count as f64).sqrt()
}

/// Returns the unit directions of the `count` plate centres of planet `seed`: a Fibonacci lattice, each
/// point moved by a seeded share of the spacing, the whole turned by a seeded rotation so that no planet
/// has a plate on its pole by rule.
pub fn centres(seed: u64, count: usize) -> Vec<DVec3> {
    let golden = PI * (3.0 - 5f64.sqrt());
    let [a, b, c, _] = rolls(seed, KEY, 1).map(f64::from);
    let turn = DQuat::from_euler(EulerRot::YXZ, TAU * a, PI * b, TAU * c);
    let reach = 2.0 * JITTER * spacing(count) / SPACING;
    (0..count)
        .map(|k| {
            let y = 1.0 - 2.0 * (k as f64 + 0.5) / count as f64;
            let (ring, angle) = ((1.0 - y * y).sqrt(), golden * k as f64);
            let at = DVec3::new(ring * angle.cos(), y, ring * angle.sin());
            let [u, v, _, _] = rolls(seed, KEY, 10 + k as u32).map(f64::from);
            let (east, north) = at.any_orthonormal_pair();
            turn * (at + (east * (u - 0.5) + north * (v - 0.5)) * reach).normalize()
        })
        .collect()
}

/// Returns the crust of each of `count` plates: the share `1 - lowland` of them continental, picked by
/// seeded rank, the rest oceanic.
pub fn crusts(seed: u64, count: usize, lowland: f64) -> Vec<Crust> {
    let lands = ((1.0 - lowland) * count as f64).round() as usize;
    let draw = |k: usize| (roll(seed, KEY, 100 + k as u32), k);
    (0..count)
        .map(|k| {
            let rank = (0..count).filter(|&j| draw(j) < draw(k)).count();
            [Crust::Oceanic, Crust::Continental][usize::from(rank < lands)]
        })
        .collect()
}

/// Returns the turn of each of `count` plates: a vector along the axis the plate turns about, 0.4 to 1
/// long, the fastest plate being 1. A point `p` of the plate moves along `spin x p`.
pub fn spins(seed: u64, count: usize) -> Vec<DVec3> {
    (0..count)
        .map(|k| {
            let [u, v, rate, _] = rolls(seed, KEY, 200 + k as u32).map(f64::from);
            let (z, angle) = (2.0 * u - 1.0, TAU * v);
            let ring = (1.0 - z * z).sqrt();
            DVec3::new(ring * angle.cos(), z, ring * angle.sin()) * (0.4 + 0.6 * rate)
        })
        .collect()
}
