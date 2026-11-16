//! Random wandering between nearby points, shared by fireflies and butterflies.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::env::Ctx;
use crate::world::SEA_LEVEL;

/// Where a small flyer is going and what it is doing.
#[derive(Component, Debug, Clone, Copy)]
pub struct Wander {
    /// Point it is flying to.
    pub target: Vec3,
    /// Direction of travel on the XZ plane.
    pub heading: f32,
    /// Seconds left to rest at the target.
    pub wait: f32,
    /// Personal phase for blinking or flapping.
    pub phase: f32,
    /// Fade from 0 (gone) to 1 (fully there).
    pub shown: f32,
    /// True when the target is a landing spot.
    pub land: bool,
    /// True while resting on the ground.
    pub landed: bool,
    /// Position in its pool.
    pub slot: usize,
}

impl Wander {
    /// Returns a fresh flyer parked far away, to be placed on its first update.
    pub fn new(slot: usize) -> Self {
        Self {
            target: Vec3::ZERO,
            heading: 0.0,
            wait: 0.0,
            phase: slot as f32 * 0.83,
            shown: 0.0,
            land: false,
            landed: false,
            slot,
        }
    }
}

/// Where a flyer waits before its first placement, far from any camera.
pub const PARKED: Vec3 = Vec3::new(1.0e6, 0.0, 1.0e6);

/// How a kind of flyer roams.
#[derive(Debug, Clone, Copy)]
pub struct Roam {
    /// Distance from the hub beyond which the next target pulls back to it.
    pub radius: f32,
    /// Farthest next target from the current spot.
    pub reach: f32,
    /// Distance from the camera beyond which it is moved back near its hub.
    pub leash: f32,
    /// Lowest and highest flight height above the ground.
    pub band: (f32, f32),
    /// Shortest and longest rest at a target.
    pub pause: (f32, f32),
    /// Chance that a target is a landing spot.
    pub land: f32,
    /// Cruise speed in meters per second.
    pub speed: f32,
}

/// Returns the ground height at `(x, z)`, with the sea surface as a floor.
pub fn ground(ctx: &Ctx, x: f32, z: f32) -> f32 {
    ctx.terrain.height(x, z).max(SEA_LEVEL)
}

/// Picks the next target near `from`, or near `hub` when too far from it.
pub fn pick(wander: &mut Wander, from: Vec3, hub: Vec3, roam: &Roam, ctx: &mut Ctx) {
    let away = Vec2::new(from.x - hub.x, from.z - hub.z).length() > roam.radius;
    let base = if away { hub } else { from };
    let angle = ctx.rng.range(0.0, TAU);
    let reach = ctx.rng.range(0.3, 1.0) * roam.reach;
    let (x, z) = (base.x + angle.cos() * reach, base.z + angle.sin() * reach);
    wander.land = ctx.rng.next() < roam.land;
    let up = if wander.land {
        0.06
    } else {
        ctx.rng.range(roam.band.0, roam.band.1)
    };
    wander.target = Vec3::new(x, ground(ctx, x, z) + up, z);
}

/// Returns a random flight point around `hub` for a flyer that strayed or just appeared.
pub fn place(wander: &mut Wander, hub: Vec3, roam: &Roam, ctx: &mut Ctx) -> Vec3 {
    let angle = ctx.rng.range(0.0, TAU);
    let reach = ctx.rng.range(0.2, 0.9) * roam.radius;
    let (x, z) = (hub.x + angle.cos() * reach, hub.z + angle.sin() * reach);
    let at = Vec3::new(
        x,
        ground(ctx, x, z) + ctx.rng.range(roam.band.0, roam.band.1),
        z,
    );
    wander.target = at;
    wander.wait = 0.0;
    wander.landed = false;
    at
}

/// Returns true when `at` lies farther than the leash from the camera on the XZ plane.
pub fn strayed(at: Vec3, camera: Vec3, roam: &Roam) -> bool {
    Vec2::new(at.x - camera.x, at.z - camera.z).length() > roam.leash
}
