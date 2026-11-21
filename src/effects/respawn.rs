//! Gives a pooled particle a new kind and a new place in the box around the camera.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::ground::Ground;
use super::kind::{Ambient, Kind};
use super::planet::{drift, seen};
use super::pool::{ABOVE, BELOW, HALF_WIDTH, Particle, Pool};
use super::rng::Rng;
use super::shed::{Shedding, start};
use super::sky::{Sky, fall};
use super::tune::{HEAD_ON, HEAD_ON_SHARE};
use crate::render::smoothstep;
use crate::universe::PlanetRecord;
use crate::world::SEA_LEVEL;

/// Closest a fresh particle may appear to the camera, in meters.
const NEAREST: f32 = 1.5;

/// Chance that a respawn happens on the upwind side of the box, where the air comes from.
const UPWIND_SHARE: f32 = 0.6;

/// Everything a respawn reads or advances.
pub struct Spawn<'a> {
    /// What surrounds the camera.
    pub ambient: Ambient,
    /// The record of the planet, which says what its air and its liquid can carry.
    pub record: PlanetRecord,
    /// What the weather drops around the camera.
    pub sky: Sky,
    /// The plants within reach, which shed the leaves, petals and pollen.
    pub shed: &'a Shedding,
    /// The ground around the camera, for heights along the local vertical.
    pub ground: Ground<'a>,
    /// Velocity in metres per second of the air against the camera, along the local horizontal:
    /// the wind less the velocity of the camera. It is the wind itself when the camera is still.
    pub flow: Vec2,
    /// Shared meshes and materials.
    pub pool: &'a Pool,
    /// Random numbers.
    pub rng: &'a mut Rng,
}

/// Returns the chance of a respawn on the upwind face for air that meets the camera at `speed`
/// metres per second: more of them the faster it does, so a body in fast flight flies into
/// particles that stream past it.
pub fn upwind_share(speed: f32) -> f32 {
    let fast = smoothstep(HEAD_ON.0, HEAD_ON.1, speed);
    UPWIND_SHARE + (1.0 - UPWIND_SHARE) * HEAD_ON_SHARE * fast
}

impl Spawn<'_> {
    /// Returns a horizontal offset in the box, often on the upwind face once the pool is primed.
    fn offset(&mut self, primed: bool) -> Vec2 {
        let mut at = Vec2::new(
            self.rng.range(-HALF_WIDTH, HALF_WIDTH),
            self.rng.range(-HALF_WIDTH, HALF_WIDTH),
        );
        let upwind = -self.flow.normalize_or_zero();
        if primed && self.rng.next() < upwind_share(self.flow.length()) {
            at += upwind * (HALF_WIDTH * 0.95 - at.dot(upwind));
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
        let floor = ground.max(self.ground.sea_level());
        if motion.low {
            return floor + self.rng.range(0.3, 2.0);
        }
        let rise = self.rng.next().sqrt();
        (eye - BELOW + (BELOW + ABOVE) * rise).max(floor + 0.3)
    }

    /// Puts a visible particle of `kind` in the box, or at `local` when it comes from a plant, and
    /// returns the height it dies at. This is the one terrain lookup of a respawn.
    fn settle(
        &mut self,
        kind: Kind,
        primed: bool,
        local: Option<Vec3>,
        transform: &mut Transform,
    ) -> f32 {
        let from_plant = local.is_some();
        let local = local.unwrap_or_else(|| self.offset(primed).extend(0.0).xzy());
        let spot = self.ground.eye.at + self.ground.turn(Vec3::new(local.x, 0.0, local.z));
        let bed = self.ground.height(spot);
        let level = match from_plant {
            true => {
                (self.ground.eye.altitude + local.y).max(bed.max(self.ground.sea_level()) + 0.3)
            }
            false => self.height(kind, bed),
        };
        transform.translation = self.ground.raised(spot, level);
        match kind.is_submerged() {
            true => bed,
            false => bed.max(self.ground.sea_level()),
        }
    }
}

/// Returns the kind of a respawning particle: what the weather drops first, then what the
/// surroundings call for. Under a clear sky it draws the same one roll as before the weather.
fn pick(spawn: &mut Spawn) -> Kind {
    let weather = match spawn.sky.falling() {
        true => fall(
            &spawn.sky,
            &seen(&spawn.record, &spawn.ambient),
            spawn.rng.next(),
        ),
        false => None,
    };
    weather.unwrap_or_else(|| drift(&spawn.record, &spawn.ambient, spawn.rng.next()))
}

/// Returns the kind of a respawning particle, the plant it comes from and where in the box it starts:
/// a leaf, petal or pollen speck with no plant in the box to shed it stays hidden.
fn sourced(spawn: &mut Spawn) -> (Kind, Option<usize>, Option<Vec3>) {
    let kind = pick(spawn);
    if !kind.is_shed() {
        return (kind, None, None);
    }
    let Some(i) = spawn.shed.choose(kind, spawn.rng.next()) else {
        return (Kind::Hidden, None, None);
    };
    let source = spawn.shed.sources[i];
    let foot = spawn.ground.span(
        spawn.ground.eye.at,
        spawn.ground.anchor.to_render(source.at),
    );
    match start(&source, foot, kind, spawn.rng) {
        Some(at) => (kind, Some(i), Some(at)),
        None => (Kind::Hidden, None, None),
    }
}

/// Gives a placed particle of `kind` its mesh and its material: the colours of its plant `from`
/// for a leaf or a petal.
fn dress(
    look: (&mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>),
    kind: Kind,
    from: Option<usize>,
    spawn: &mut Spawn,
) {
    let mesh = spawn.pool.mesh(kind);
    if look.0.0 != *mesh {
        look.0.0 = mesh.clone();
    }
    let material = match (kind, from) {
        (Kind::Leaf, Some(i)) => &spawn.pool.leaves[i],
        (Kind::Petal, Some(i)) => &spawn.pool.petals[i],
        _ => spawn.pool.material(kind, (spawn.rng.next() * 8.0) as usize),
    };
    if look.1.0 != *material {
        look.1.0 = material.clone();
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
    let (kind, from, spot) = sourced(spawn);
    let source = from.and_then(|i| spawn.shed.sources.get(i)).copied();
    let motion = kind.motion();
    transform.scale = Vec3::ZERO;
    particle.kind = kind;
    particle.age = 0.0;
    particle.life = spawn.rng.range(motion.life.0, motion.life.1);
    particle.phase = spawn.rng.range(0.0, TAU);
    particle.size = match (kind, source) {
        (Kind::Leaf, Some(source)) => source.shed.size,
        _ => motion.size,
    };
    let primed = std::mem::replace(&mut particle.primed, true);
    if kind == Kind::Hidden {
        return;
    }
    particle.floor = spawn.settle(kind, primed, spot, transform);
    dress(look, kind, from, spawn);
}
