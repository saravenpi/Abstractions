//! How wet the canopy is and how its drops move: pure rules of the drops, read by `drops`, and
//! where on the glass each drop starts.
//!
//! The glass wets in cloud and in rain and dries once out of them, faster in a fast airflow
//! that blows the water off. At rest the drops bead and creep down the glass; at speed the air
//! drags them back over the glass toward the tail, drawn out into streaks.

use bevy::prelude::*;

use super::tune::HINGE;
use crate::render::smoothstep;
use crate::ship::mesh::glass;

/// Drops on the glass at most.
pub const DROPS: usize = 96;

/// Metres a drop runs over its pane before another takes its place at the start.
pub const TRAVEL: f32 = 0.35;

/// Half width of the smallest and of the largest drop, in metres.
const SIZE: (f32, f32) = (0.004, 0.009);

/// Metres a drop sits out from the glass, so the glass never hides it.
pub const LIFT: f32 = 0.004;

/// Seconds in which the glass takes most of the wetness of the air it flies through.
const SOAK: f32 = 1.5;

/// Seconds in which wet glass dries out of the cloud: at rest, and what a fast airflow takes
/// off it, so the glass clears in a few seconds at speed.
const DRY: (f32, f32) = (12.0, 1.2);

/// Speed in metres per second of the airflow at which it dries the glass half way from the rest
/// to the fastest.
const BLOW: f32 = 20.0;

/// Airspeeds in metres per second between which the drops pass from beading under their weight
/// to being dragged back by the airflow.
const DRAG: (f32, f32) = (6.0, 25.0);

/// Speed in metres per second a bead creeps down the glass, the share of the airspeed a
/// streaming drop runs back at, and the fastest it runs.
const RUN: (f32, f32, f32) = (0.02, 0.03, 2.5);

/// How many times longer than wide a drop is at the airspeed of [`STRETCH`].1, and that speed.
const STRETCH: (f32, f32) = (4.0, 120.0);

/// Returns the wetness of the glass, from 0 to 1, `dt` seconds after `wet`, flying through air
/// as wet as `soak` from 0 to 1 at `speed` metres per second.
pub fn wetted(wet: f32, soak: f32, speed: f32, dt: f32) -> f32 {
    let soak = soak.clamp(0.0, 1.0);
    let drying = DRY.0 + (DRY.1 - DRY.0) * speed.max(0.0) / (speed.max(0.0) + BLOW);
    let rate = if soak > wet { 1.0 / SOAK } else { 1.0 / drying };
    let next = soak + (wet - soak) * (-rate * dt.max(0.0)).exp();
    if soak == 0.0 && next < 1.0e-3 {
        0.0
    } else {
        next
    }
}

/// Returns how far the airflow at `speed` metres per second drags the drops, from 0 when they
/// bead to 1 when it streams them back.
pub fn dragged(speed: f32) -> f32 {
    smoothstep(DRAG.0, DRAG.1, speed)
}

/// Returns the speed in metres per second a drop moves over the glass at the airspeed `speed`.
pub fn run(speed: f32) -> f32 {
    let streaming = (RUN.1 * speed).min(RUN.2);
    RUN.0 + (streaming - RUN.0) * dragged(speed)
}

/// Returns how many times longer than wide a drop is at the airspeed `speed`.
pub fn stretch(speed: f32) -> f32 {
    1.0 + (STRETCH.0 - 1.0) * dragged(speed) * (speed / STRETCH.1).min(1.0)
}

/// Returns the unit direction a drop moves in over a pane facing `normal`, in hull axes: down
/// the glass under its weight at rest, back toward the tail in the airflow at `speed`.
pub fn way(normal: Vec3, speed: f32) -> Vec3 {
    let flat = |v: Vec3| (v - normal * v.dot(normal)).normalize_or_zero();
    let pulled = flat(Vec3::NEG_Y).lerp(flat(Vec3::Z), dragged(speed));
    pulled.try_normalize().unwrap_or(flat(Vec3::Z))
}

/// Where one drop starts, in the axes of the hinge of the canopy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seed {
    /// A point of the glass.
    pub at: Vec3,
    /// The outward unit normal of its pane.
    pub normal: Vec3,
    /// Where along its run it starts, from 0 to 1.
    pub phase: f32,
    /// Its half width in metres.
    pub size: f32,
    /// The wetness of the glass from which it shows, from 0 to 1.
    pub order: f32,
}

/// Returns a fixed number in `[0, 1)` for `slot` and `salt`.
fn hash(slot: u32, salt: u32) -> f32 {
    let mut x = slot.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    (x >> 8) as f32 / 16_777_216.0
}

/// Returns the seeds of every drop, each on a pane of the canopy glass.
pub fn seeds() -> Vec<Seed> {
    let panes: Vec<[Vec3; 3]> = glass()
        .tris
        .iter()
        .map(|tri| tri.map(|p| p - HINGE))
        .collect();
    (0..DROPS as u32)
        .map(|slot| {
            let [a, b, c] = panes[(hash(slot, 1) * panes.len() as f32) as usize % panes.len()];
            let (mut u, mut v) = (hash(slot, 2), hash(slot, 3));
            if u + v > 1.0 {
                (u, v) = (1.0 - u, 1.0 - v);
            }
            Seed {
                at: a + (b - a) * u + (c - a) * v,
                normal: (b - a).cross(c - a).normalize_or(Vec3::Y),
                phase: hash(slot, 4),
                size: SIZE.0 + (SIZE.1 - SIZE.0) * hash(slot, 5),
                order: hash(slot, 6),
            }
        })
        .collect()
}

/// Returns the middle of the drop of `seed` and its unit direction of run, `seconds` into the
/// flight at the airspeed `speed`.
pub fn placed(seed: &Seed, seconds: f32, speed: f32) -> (Vec3, Vec3) {
    let along = way(seed.normal, speed);
    let gone = (seed.phase + seconds * run(speed) / TRAVEL).fract() - 0.5;
    (
        seed.at + along * (gone * TRAVEL) + seed.normal * LIFT,
        along,
    )
}
