//! Day birds: crows over land and gulls over the sea, flying in loose flocks high above the ground.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::anim::{glide, wing_angle};
use super::env::{Ctx, Env, Sky};
use super::flock::{CAPACITY, Flock, Flocks, formation};
use super::gear::Gear;
use super::habitat::{Species, presence_on};
use super::steer::{approach, yaw};
use super::wander::PARKED;
use super::wings::{Spec, Wing, pair, pose};
use crate::screens::Screen;
use crate::space::Rebased;

/// A bird in a flock.
#[derive(Component, Debug, Clone, Copy)]
pub struct Bird {
    /// Which flock it belongs to: 0 for crows, 1 for gulls.
    flock: usize,
    /// Place in the formation.
    slot: usize,
    /// Fade from 0 (gone) to 1 (fully there).
    shown: f32,
}

/// Spawns both flocks with two wings per bird. Birds are free entities in render space, so
/// each body is marked to follow anchor rebases; the wings are its children.
pub fn spawn(commands: &mut Commands, gear: &Gear) {
    for (flock, capacity) in CAPACITY.into_iter().enumerate() {
        let feathers = if flock == 0 { &gear.crow } else { &gear.gull };
        for slot in 0..capacity {
            let wings = Spec {
                mesh: gear.wing.clone(),
                material: feathers.clone(),
                length: 0.9,
                width: 0.4,
                hinge: 0.1,
            };
            commands.spawn((
                Bird {
                    flock,
                    slot,
                    shown: 0.0,
                },
                Mesh3d(gear.bird_body.clone()),
                MeshMaterial3d(feathers.clone()),
                Transform::from_translation(PARKED).with_scale(Vec3::ZERO),
                Rebased,
                NotShadowCaster,
                DespawnOnEnter(Screen::Title),
                pair(&wings),
            ));
        }
    }
}

/// Moves a bird toward its place in the flock and returns the wing angle to show.
fn fly(bird: &mut Bird, at: &mut Transform, flock: &Flock, level: f32, sky: &Sky) -> f32 {
    let dt = sky.dt;
    let want = (level - bird.slot as f32).clamp(0.0, 1.0);
    bird.shown = approach(bird.shown, want, dt * 0.5);
    let goal = flock.center + sky.tilt * formation(bird.slot, flock.heading);
    if at.translation.distance(goal) > 40.0 {
        at.translation = goal;
    } else {
        at.translation = at.translation.lerp(goal, 1.0 - (-2.5 * dt).exp());
    }
    let bank = (flock.turn * 2.0).clamp(-0.6, 0.6);
    let level_flight = Quat::from_rotation_y(yaw(flock.heading)) * Quat::from_rotation_z(-bank);
    at.rotation = sky.tilt * level_flight;
    at.scale = Vec3::splat(bird.shown);
    let phase = sky.now * TAU * 3.0 + bird.slot as f32 * 0.7;
    let seed = bird.flock as f32 * 0.3 + bird.slot as f32 * 0.05;
    wing_angle(phase, glide(sky.now, seed, 0.35), 0.6, 0.1)
}

/// Flies both flocks and every bird in them. A flock that is neither wanted nor shown rests:
/// it is not moved and reads no terrain.
pub fn step(
    mut env: Env,
    mut flocks: ResMut<Flocks>,
    mut birds: Query<(&mut Bird, &mut Transform, &Children), Without<Wing>>,
    mut wings: Query<(&mut Transform, &Wing)>,
) {
    let sky = env.sky();
    let globe = env.globe();
    let levels =
        [Species::Crow, Species::Gull].map(|species| presence_on(&globe, species, &sky.ambient));
    let mut rng = *env.rng;
    let mut ctx = Ctx {
        ground: env.ground(),
        rng: &mut rng,
    };
    for (index, flock) in flocks.0.iter_mut().enumerate() {
        if levels[index] > 0.0 || flock.live {
            flock.advance(index, &sky, &mut ctx);
        }
        flock.live = false;
    }
    for (mut bird, mut at, children) in &mut birds {
        let flock = flocks.0[bird.flock];
        let level = levels[bird.flock] * flock.size as f32;
        if level <= 0.0 && bird.shown <= 0.0 {
            continue;
        }
        let angle = fly(&mut bird, &mut at, &flock, level, &sky);
        pose(&mut wings, children, angle);
        flocks.0[bird.flock].live |= bird.shown > 0.0;
    }
    *env.rng = rng;
}
