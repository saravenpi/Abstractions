//! Random wandering between nearby points, shared by fireflies and butterflies.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::env::Ctx;

/// Where a small flyer is going and what it is doing.
#[derive(Component, Debug, Clone, Copy)]
pub struct Wander {
    /// Point it is flying to, in render space.
    pub target: Vec3,
    /// Direction of travel on the local XZ plane of the camera.
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
    /// Distance from the hub beyond which the next targets lead back toward it.
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

/// Returns the height above sea level of the ground under render position `at`, with the sea
/// surface as a floor where the planet has a sea. This samples the terrain.
pub fn ground(ctx: &Ctx, at: Vec3) -> f32 {
    ctx.ground.height(at).max(ctx.ground.sea_level())
}

/// Returns the render position `reach` meters from `base` along local heading `angle`, lifted
/// to `above` meters over the ground there.
fn spot(ctx: &Ctx, base: Vec3, (angle, reach): (f32, f32), above: f32) -> Vec3 {
    let flat = base
        + ctx
            .ground
            .turn(Vec3::new(angle.cos(), 0.0, angle.sin()) * reach);
    ctx.ground.raised(flat, ground(ctx, flat) + above)
}

/// Picks the next target within reach of `from`: anywhere around it, or back toward `hub`
/// when too far from it. Every leg is short, so a straight flight stays near the ground.
pub fn pick(wander: &mut Wander, from: Vec3, hub: Vec3, roam: &Roam, ctx: &mut Ctx) {
    let home = ctx.ground.span(from, hub);
    let home = Vec2::new(home.x, home.z);
    let angle = match home.length() > roam.radius {
        true => home.to_angle() + ctx.rng.range(-0.6, 0.6),
        false => ctx.rng.range(0.0, TAU),
    };
    let reach = ctx.rng.range(0.3, 1.0) * roam.reach;
    wander.land = ctx.rng.next() < roam.land;
    let above = if wander.land {
        0.06
    } else {
        ctx.rng.range(roam.band.0, roam.band.1)
    };
    wander.target = spot(ctx, from, (angle, reach), above);
}

/// Returns a random flight point around `hub` for a flyer that strayed or just appeared.
pub fn place(wander: &mut Wander, hub: Vec3, roam: &Roam, ctx: &mut Ctx) -> Vec3 {
    let angle = ctx.rng.range(0.0, TAU);
    let reach = ctx.rng.range(0.2, 0.9) * roam.radius;
    let above = ctx.rng.range(roam.band.0, roam.band.1);
    let at = spot(ctx, hub, (angle, reach), above);
    wander.target = at;
    wander.wait = 0.0;
    wander.landed = false;
    at
}

/// Returns true when `at` lies farther than the leash from the camera, measured along the ground.
pub fn strayed(at: Vec3, ctx: &Ctx, roam: &Roam) -> bool {
    let off = ctx.ground.span(ctx.ground.eye.at, at);
    Vec2::new(off.x, off.z).length() > roam.leash
}

/// Returns true when a flyer of the pool is neither shown nor wanted: `level` flyers should be
/// out. Such a flyer is left alone, so it costs no terrain lookup while it is out of sight.
pub fn dormant(wander: &Wander, level: f32) -> bool {
    wander.shown <= 0.0 && level <= wander.slot as f32
}
