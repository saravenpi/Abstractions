//! Moves every particle each frame: wind drift, fall, flutter, tumble, fades and respawns.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::creatures::anim::ramp;
use super::eye::Eye;
use super::ground::Ground;
use super::kind::{Ambient, Kind};
use super::pool::{ABOVE, BELOW, HALF_WIDTH, Particle, Pool};
use super::respawn::{Spawn, respawn};
use super::rng::Rng;
use super::sense::Sensed;
use crate::render::{DayClock, daylight};
use crate::space::Anchor;
use crate::wind::Wind;
use crate::world::{SEA_LEVEL, Terrain};

/// Most respawns, and so terrain lookups, in one frame.
const MAX_RESPAWNS: usize = 24;

/// Resources the simulation reads.
#[derive(SystemParam)]
pub struct Env<'w> {
    time: Res<'w, Time>,
    wind: Res<'w, Wind>,
    sensed: Res<'w, Sensed>,
    clock: Res<'w, DayClock>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    eye: Res<'w, Eye>,
    pool: Res<'w, Pool>,
    rng: ResMut<'w, Rng>,
}

/// Returns true when the particle left the box around the camera, hit the ground or left the
/// water. The box and the heights follow the local vertical.
fn gone(particle: &Particle, position: Vec3, ground: &Ground) -> bool {
    let off = ground.span(ground.eye.at, position);
    let outside = off.x.abs() > HALF_WIDTH
        || off.z.abs() > HALF_WIDTH
        || off.y < -BELOW - 1.0
        || off.y > ABOVE + 1.0;
    let level = ground.altitude(position);
    let ended = match particle.kind.is_submerged() {
        true => level > SEA_LEVEL,
        false => level < particle.floor,
    };
    outside || ended
}

/// Returns the draw scale of a particle from its size and fades.
fn scale_of(particle: &Particle, distance: f32) -> Vec3 {
    let size = particle.kind.motion().size;
    let grow = (particle.age * 2.0)
        .min((particle.life - particle.age) * 2.0)
        .clamp(0.0, 1.0);
    let s = size * grow * ramp(0.6, 1.2, distance);
    match particle.kind.is_leafy() {
        true => Vec3::new(s * 0.55, s, 1.0),
        false => Vec3::new(s, s, 1.0),
    }
}

/// Advances one live particle by `dt` seconds: the wind blows along the local horizontal and
/// the fall follows the local vertical.
fn drift(particle: &mut Particle, transform: &mut Transform, env: &Env) {
    let dt = env.time.delta_secs();
    let motion = particle.kind.motion();
    particle.age += dt;
    let t = particle.phase + particle.age * motion.rate;
    let gust = 0.6 + env.wind.strength;
    let flutter =
        Vec3::new(t.sin(), 0.4 * (1.7 * t).cos(), (0.8 * t + 1.0).cos()) * motion.sway * gust;
    let velocity = env.wind.velocity() * motion.wind + flutter - Vec3::Y * motion.fall;
    transform.translation += env.eye.tilt * velocity * dt;
    transform.scale = scale_of(particle, transform.translation.distance(env.eye.at));
    transform.rotation = match motion.spin > 0.0 {
        true => {
            let spin = particle.phase + particle.age * motion.spin;
            Quat::from_euler(EulerRot::XYZ, spin, spin * 0.7 + 1.7, spin * 0.4 + 0.3)
        }
        false => env.eye.rot,
    };
}

/// Ages a hidden particle or drifts a visible one.
fn advance(particle: &mut Particle, transform: &mut Transform, env: &Env) {
    match particle.kind {
        Kind::Hidden => particle.age += env.time.delta_secs(),
        _ => drift(particle, transform, env),
    }
}

/// Returns the surroundings of the camera.
fn ambient_of(env: &Env, height: f32) -> Ambient {
    Ambient {
        biome: env.sensed.biome,
        surface: env.sensed.surface,
        height,
        daylight: daylight(env.clock.phase),
        wind: env.wind.strength,
    }
}

/// Moves the whole pool around the eye of this frame.
pub fn step(
    mut env: Env,
    mut particles: Query<(
        &mut Particle,
        &mut Transform,
        &mut Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let ground = Ground::new(&env.anchor, &env.terrain, &env.eye);
    let mut budget = MAX_RESPAWNS;
    let mut rng = *env.rng;
    let mut spawn = Spawn {
        ambient: ambient_of(&env, env.eye.altitude),
        ground,
        wind: env.wind.direction,
        pool: &env.pool,
        rng: &mut rng,
    };
    for (mut particle, mut transform, mut mesh, mut material) in &mut particles {
        let due = !particle.primed
            || particle.age >= particle.life
            || gone(&particle, transform.translation, &ground);
        if !due {
            advance(&mut particle, &mut transform, &env);
        } else if budget > 0 {
            budget -= 1;
            respawn(
                &mut particle,
                &mut transform,
                (&mut mesh, &mut material),
                &mut spawn,
            );
        } else if transform.scale != Vec3::ZERO {
            transform.scale = Vec3::ZERO;
        }
    }
    *env.rng = rng;
}
