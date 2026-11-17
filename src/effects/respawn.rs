//! Gives a pooled particle a new kind and a new place in the box around the camera.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::ground::Ground;
use super::kind::{Ambient, Kind};
use super::pick::pick;
use super::pool::{ABOVE, BELOW, HALF_WIDTH, Particle, Pool};
use super::rng::Rng;
use crate::world::SEA_LEVEL;

/// Closest a fresh particle may appear to the camera, in meters.
const NEAREST: f32 = 1.5;

/// Chance that a respawn happens on the upwind side of the box.
const UPWIND_SHARE: f32 = 0.6;

/// Everything a respawn reads or advances.
pub struct Spawn<'a> {
    /// What surrounds the camera.
    pub ambient: Ambient,
    /// The ground around the camera, for heights along the local vertical.
    pub ground: Ground<'a>,
    /// Unit wind direction in the local frame of the camera.
    pub wind: Vec2,
    /// Shared meshes and materials.
    pub pool: &'a Pool,
    /// Random numbers.
    pub rng: &'a mut Rng,
}

impl Spawn<'_> {
    /// Returns a horizontal offset in the box, often on the upwind face once the pool is primed.
    fn offset(&mut self, primed: bool) -> Vec2 {
        let mut at = Vec2::new(
            self.rng.range(-HALF_WIDTH, HALF_WIDTH),
            self.rng.range(-HALF_WIDTH, HALF_WIDTH),
        );
        if primed && self.rng.next() < UPWIND_SHARE {
            at += self.wind * (-HALF_WIDTH * 0.95 - at.dot(self.wind));
            at = at.clamp(
                Vec2::splat(-HALF_WIDTH * 0.95),
                Vec2::splat(HALF_WIDTH * 0.95),
            );
        }
        if at.length_squared() < NEAREST * NEAREST {
            at.x += NEAREST;
        }
        at
    }

    /// Returns the spawn height above sea level for a kind over ground at `ground`.
    fn height(&mut self, kind: Kind, ground: f32) -> f32 {
        let motion = kind.motion();
        let eye = self.ground.eye.altitude;
        let top = eye + ABOVE;
        if kind.is_submerged() {
            let high = top.min(SEA_LEVEL - 0.3);
            let low = ground.max(eye - BELOW).min(high - 0.1);
            return self.rng.range(low, high);
        }
        let floor = ground.max(SEA_LEVEL);
        if motion.low {
            return floor + self.rng.range(0.3, 2.0);
        }
        let rise = self.rng.next().sqrt();
        (eye - BELOW + (BELOW + ABOVE) * rise).max(floor + 0.3)
    }

    /// Puts a visible particle of `kind` in the box and returns the height it dies at. This is
    /// the one terrain lookup of a respawn.
    fn settle(&mut self, kind: Kind, primed: bool, transform: &mut Transform) -> f32 {
        let at = self.offset(primed);
        let spot = self.ground.eye.at + self.ground.turn(Vec3::new(at.x, 0.0, at.y));
        let bed = self.ground.height(spot);
        let level = self.height(kind, bed);
        transform.translation = self.ground.raised(spot, level);
        match kind.is_submerged() {
            true => bed,
            false => bed.max(SEA_LEVEL),
        }
    }
}

/// Re-rolls a particle's kind, position, life and look. A hidden particle only gets a new life:
/// it is not drawn, so it costs no terrain lookup.
pub fn respawn(
    particle: &mut Particle,
    transform: &mut Transform,
    look: (&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>),
    spawn: &mut Spawn,
) {
    let kind = pick(&spawn.ambient, spawn.rng.next());
    let motion = kind.motion();
    transform.scale = Vec3::ZERO;
    particle.kind = kind;
    particle.age = 0.0;
    particle.life = spawn.rng.range(motion.life.0, motion.life.1);
    particle.phase = spawn.rng.range(0.0, TAU);
    let primed = std::mem::replace(&mut particle.primed, true);
    if kind == Kind::Hidden {
        return;
    }
    particle.floor = spawn.settle(kind, primed, transform);
    let mesh = spawn.pool.mesh(kind);
    if look.0.0 != *mesh {
        look.0.0 = mesh.clone();
    }
    let material = spawn.pool.material(kind, (spawn.rng.next() * 8.0) as usize);
    if look.1.0 != *material {
        look.1.0 = material.clone();
    }
}
