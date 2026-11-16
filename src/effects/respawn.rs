//! Gives a pooled particle a new kind and a new place in the box around the camera.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::kind::{Ambient, Kind};
use super::pick::pick;
use super::pool::{ABOVE, BELOW, HALF_WIDTH, Particle, Pool};
use super::rng::Rng;
use crate::world::{SEA_LEVEL, Terrain};

/// Closest a fresh particle may appear to the camera, in meters.
const NEAREST: f32 = 1.5;

/// Chance that a respawn happens on the upwind side of the box.
const UPWIND_SHARE: f32 = 0.6;

/// Everything a respawn reads or advances.
pub struct Spawn<'a> {
    /// What surrounds the camera.
    pub ambient: Ambient,
    /// Camera position.
    pub camera: Vec3,
    /// Unit wind direction on the XZ plane.
    pub wind: Vec2,
    /// Terrain for ground heights.
    pub terrain: &'a Terrain,
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

    /// Returns the spawn height for a kind over ground at `ground`.
    fn height(&mut self, kind: Kind, ground: f32) -> f32 {
        let motion = kind.motion();
        let top = self.camera.y + ABOVE;
        if kind.is_submerged() {
            let high = top.min(SEA_LEVEL - 0.3);
            let low = ground.max(self.camera.y - BELOW).min(high - 0.1);
            return self.rng.range(low, high);
        }
        let floor = ground.max(SEA_LEVEL);
        if motion.low {
            return floor + self.rng.range(0.3, 2.0);
        }
        let rise = self.rng.next().sqrt();
        (self.camera.y - BELOW + (BELOW + ABOVE) * rise).max(floor + 0.3)
    }
}

/// Re-rolls a particle's kind, position, life and look.
pub fn respawn(
    particle: &mut Particle,
    transform: &mut Transform,
    look: (&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>),
    spawn: &mut Spawn,
) {
    let kind = pick(&spawn.ambient, spawn.rng.next());
    let motion = kind.motion();
    let at = spawn.offset(particle.primed);
    let ground = spawn
        .terrain
        .height(spawn.camera.x + at.x, spawn.camera.z + at.y);
    let y = spawn.height(kind, ground);
    transform.translation = Vec3::new(spawn.camera.x + at.x, y, spawn.camera.z + at.y);
    transform.scale = Vec3::ZERO;
    particle.kind = kind;
    particle.age = 0.0;
    particle.life = spawn.rng.range(motion.life.0, motion.life.1);
    particle.phase = spawn.rng.range(0.0, TAU);
    particle.floor = if kind.is_submerged() {
        ground
    } else {
        ground.max(SEA_LEVEL)
    };
    particle.primed = true;
    let mesh = spawn.pool.mesh(kind);
    if look.0.0 != *mesh {
        look.0.0 = mesh.clone();
    }
    let material = spawn.pool.material(kind, (spawn.rng.next() * 8.0) as usize);
    if look.1.0 != *material {
        look.1.0 = material.clone();
    }
}
