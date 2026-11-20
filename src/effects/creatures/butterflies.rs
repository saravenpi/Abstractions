//! Day butterflies: erratic fluttering between nearby points, with the odd rest on the ground.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::anim::{glide, wing_angle};
use super::env::{Ctx, Env, Sky};
use super::gear::Gear;
use super::habitat::{Species, presence_on};
use super::steer::{approach, bearing, turn_toward, yaw};
use super::wander::{PARKED, Roam, Wander, dormant, pick, place, strayed};
use super::wings::{Spec, Wing, pair, pose};
use crate::screens::Screen;
use crate::space::Rebased;

/// Butterflies in the pool.
const COUNT: usize = 12;

/// Wing angle in radians when folded up over the back.
const FOLDED: f32 = 1.3;

/// How butterflies roam around the camera.
const ROAM: Roam = Roam {
    radius: 18.0,
    reach: 6.0,
    leash: 30.0,
    band: (0.4, 2.0),
    pause: (2.0, 6.0),
    land: 0.25,
    speed: 1.5,
};

/// Flight state, transform and wings of a butterfly.
type MothData = (
    &'static mut Wander,
    &'static mut Transform,
    &'static Children,
);

/// Marks a butterfly body.
#[derive(Component, Debug, Clone, Copy)]
pub struct Butterfly;

/// Spawns the butterflies with their two wings each. They are free entities in render space,
/// so each body is marked to follow anchor rebases; the wings are its children.
pub fn spawn(commands: &mut Commands, gear: &Gear) {
    for slot in 0..COUNT {
        let wings = Spec {
            mesh: gear.wing.clone(),
            material: gear.moth_wings[slot % gear.moth_wings.len()].clone(),
            length: 0.14,
            width: 0.12,
            hinge: 0.01,
        };
        commands.spawn((
            Butterfly,
            Wander::new(slot),
            Mesh3d(gear.moth_body.clone()),
            MeshMaterial3d(gear.moth_dark.clone()),
            Transform::from_translation(PARKED).with_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
            pair(&wings),
        ));
    }
}

/// Ends a flight leg: lands when the target is a landing spot, else picks the next target.
fn arrive(wander: &mut Wander, at: &mut Transform, hub: Vec3, ctx: &mut Ctx) {
    if wander.land {
        at.translation = wander.target;
        wander.landed = true;
        wander.wait = ctx.rng.range(ROAM.pause.0, ROAM.pause.1);
    } else {
        pick(wander, at.translation, hub, &ROAM, ctx);
    }
}

/// Flutters one step along `to`, the local vector to the target, with an erratic heading.
fn flutter(wander: &mut Wander, at: &mut Transform, sky: &Sky, to: Vec3) {
    let dt = sky.dt;
    let erratic = 0.9 * (sky.now * 5.0 + wander.slot as f32 * 1.7).sin();
    let desired = bearing(Vec3::ZERO, to) + erratic;
    wander.heading = turn_toward(wander.heading, desired, 4.0 * dt);
    let speed = ROAM.speed.min(Vec2::new(to.x, to.z).length() * 2.0 + 0.4);
    let dir = Vec2::from_angle(wander.heading) * speed * dt;
    let climb = approach(0.0, to.y, 0.8 * dt) + 0.4 * wander.phase.cos() * dt;
    at.translation += sky.tilt * Vec3::new(dir.x, climb, dir.y);
    at.rotation = sky.tilt * Quat::from_rotation_y(yaw(wander.heading));
}

/// Advances one butterfly and returns the wing angle to show.
fn live(wander: &mut Wander, at: &mut Transform, sky: &Sky, level: f32, ctx: &mut Ctx) -> f32 {
    let dt = sky.dt;
    let hub = sky.camera;
    let want = (level - wander.slot as f32).clamp(0.0, 1.0);
    wander.shown = approach(wander.shown, want, dt * 0.4);
    let rate = 8.0 + (wander.slot % 5) as f32;
    wander.phase = (wander.phase + dt * TAU * rate) % TAU;
    let to = ctx.ground.span(at.translation, wander.target);
    if strayed(at.translation, ctx, &ROAM) {
        at.translation = place(wander, hub, &ROAM, ctx);
    } else if wander.landed {
        wander.wait -= dt;
        if wander.wait <= 0.0 {
            wander.landed = false;
            pick(wander, at.translation, hub, &ROAM, ctx);
        }
    } else if Vec2::new(to.x, to.z).length() < 0.5 && to.y.abs() < 0.4 {
        arrive(wander, at, hub, ctx);
    } else {
        flutter(wander, at, sky, to);
    }
    at.scale = Vec3::splat(wander.shown);
    if wander.landed {
        wing_angle(wander.phase, 1.0, 0.0, FOLDED)
    } else {
        let gliding = glide(sky.now, wander.slot as f32 * 0.37, 0.8);
        wing_angle(wander.phase, gliding, 0.9, 0.3)
    }
}

/// Moves every butterfly that is out or wanted and flaps its wings. The others rest untouched.
pub fn step(
    mut env: Env,
    mut moths: Query<MothData, (With<Butterfly>, Without<Wing>)>,
    mut wings: Query<(&mut Transform, &Wing)>,
) {
    let sky = env.sky();
    let level = presence_on(&env.globe(), Species::Butterfly, &sky.ambient) * COUNT as f32;
    let mut rng = *env.rng;
    let mut ctx = Ctx {
        ground: env.ground(),
        rng: &mut rng,
    };
    for (mut wander, mut at, children) in &mut moths {
        if dormant(&wander, level) {
            continue;
        }
        let angle = live(&mut wander, &mut at, &sky, level, &mut ctx);
        pose(&mut wings, children, angle);
    }
    *env.rng = rng;
}
