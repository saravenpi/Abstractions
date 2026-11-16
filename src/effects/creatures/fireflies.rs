//! Night fireflies: glowing bodies wandering in loose swarms, with real lights on the nearest few.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::anim::blink;
use super::env::{Ctx, Env, Sky};
use super::gear::Gear;
use super::habitat::{Species, presence};
use super::steer::{approach, step_toward};
use super::wander::{PARKED, Roam, Wander, pick, place, strayed};
use crate::render::GameCamera;
use crate::screens::Screen;

/// Fireflies in the pool.
const COUNT: usize = 24;

/// Fireflies that carry a real light, at most.
pub const LIGHTS: usize = 6;

/// Loose swarms the fireflies gather around.
const SWARMS: usize = 4;

/// Body diameter in meters.
const SIZE: f32 = 0.09;

/// Light power in lumens at full glow.
const LIGHT_POWER: f32 = 900.0;

/// How fireflies roam around their swarm.
const ROAM: Roam = Roam {
    radius: 6.0,
    reach: 4.0,
    leash: 28.0,
    band: (0.3, 2.5),
    pause: (1.0, 4.0),
    land: 0.0,
    speed: 0.5,
};

/// Transform, light and visibility of a firefly light.
type LightData = (
    &'static mut Transform,
    &'static mut PointLight,
    &'static mut Visibility,
);

/// Marks a firefly body.
#[derive(Component, Debug, Clone, Copy)]
pub struct Firefly;

/// Marks one of the lights that follow the nearest fireflies.
#[derive(Component, Debug, Clone, Copy)]
pub struct FireflyLight;

/// Anchor points of the swarms, on the XZ plane.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Swarms([Vec3; SWARMS]);

/// The glowing fireflies closest to the camera.
struct Nearest([(f32, Vec3, f32); LIGHTS]);

impl Nearest {
    /// Keeps the firefly at `at` when it is closer than the farthest one kept.
    fn offer(&mut self, d2: f32, at: Vec3, glow: f32) {
        let worst = self
            .0
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.0.total_cmp(&b.1.0))
            .map_or(0, |(i, _)| i);
        if d2 < self.0[worst].0 {
            self.0[worst] = (d2, at, glow);
        }
    }
}

/// What a firefly update needs to know about the sky.
#[derive(Debug, Clone, Copy)]
struct Flight {
    /// The frame's view of the world.
    sky: Sky,
    /// Number of fireflies that should be out.
    level: f32,
}

/// Spawns the fireflies and their lights.
pub fn spawn(commands: &mut Commands, gear: &Gear) {
    for slot in 0..COUNT {
        commands.spawn((
            Firefly,
            Wander::new(slot),
            Mesh3d(gear.glow_mesh.clone()),
            MeshMaterial3d(gear.glow.clone()),
            Transform::from_translation(PARKED).with_scale(Vec3::ZERO),
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
    for _ in 0..LIGHTS {
        commands.spawn((
            FireflyLight,
            PointLight {
                color: Color::srgb(0.8, 1.0, 0.35),
                intensity: 0.0,
                range: 4.0,
                radius: 0.05,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::default(),
            Visibility::Hidden,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Moves swarm anchors that fell behind to a fresh spot near the camera.
fn gather(swarms: &mut Swarms, sky: &Sky, ctx: &mut Ctx) {
    for anchor in &mut swarms.0 {
        if strayed(*anchor, sky.camera, &ROAM) {
            let angle = ctx.rng.range(0.0, TAU);
            let reach = ctx.rng.range(6.0, 20.0);
            *anchor = sky.camera + Vec3::new(angle.cos() * reach, 0.0, angle.sin() * reach);
        }
    }
}

/// Advances one firefly and returns how brightly it shines.
fn fly(wander: &mut Wander, at: &mut Transform, flight: &Flight, hub: Vec3, ctx: &mut Ctx) -> f32 {
    let Flight { sky, level } = *flight;
    let dt = sky.dt;
    wander.shown = approach(
        wander.shown,
        (level - wander.slot as f32).clamp(0.0, 1.0),
        dt * 0.4,
    );
    wander.phase = (wander.phase + dt * (1.1 + 0.1 * (wander.slot % 7) as f32)) % TAU;
    if strayed(at.translation, sky.camera, &ROAM) {
        at.translation = place(wander, hub, &ROAM, ctx);
    } else if wander.wait > 0.0 {
        wander.wait -= dt;
        if wander.wait <= 0.0 {
            pick(wander, at.translation, hub, &ROAM, ctx);
        }
    } else if at.translation.distance(wander.target) < 0.2 {
        wander.wait = ctx.rng.range(ROAM.pause.0, ROAM.pause.1);
    } else {
        at.translation = step_toward(at.translation, wander.target, ROAM.speed * dt);
    }
    let p = wander.phase;
    at.translation +=
        Vec3::new((p * 0.7).sin(), 0.5 * (p * 1.3).cos(), (p * 0.9).cos()) * 0.15 * dt;
    let glow = blink(p, sky.ambient.daylight) * wander.shown;
    at.scale = Vec3::splat(SIZE * wander.shown * (0.4 + 0.6 * glow));
    glow
}

/// Moves the lights onto the nearest glowing fireflies and sets their power.
fn light_up(
    lights: &mut Query<LightData, (With<FireflyLight>, Without<Firefly>)>,
    near: &Nearest,
    dt: f32,
) {
    let follow = 1.0 - (-8.0 * dt).exp();
    for ((mut at, mut light, mut visibility), &(_, spot, glow)) in lights.iter_mut().zip(&near.0) {
        let lit = glow > 0.01;
        if *visibility == Visibility::Hidden {
            at.translation = spot;
        }
        at.translation = at.translation.lerp(spot, follow);
        light.intensity = LIGHT_POWER * glow;
        visibility.set_if_neq(if lit {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}

/// Moves the swarms, the fireflies and their lights.
pub fn step(
    mut env: Env,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    mut swarms: ResMut<Swarms>,
    mut flies: Query<(&mut Wander, &mut Transform), With<Firefly>>,
    mut lights: Query<LightData, (With<FireflyLight>, Without<Firefly>)>,
) {
    let sky = env.sky(camera.translation());
    let flight = Flight {
        sky,
        level: presence(Species::Firefly, &sky.ambient) * COUNT as f32,
    };
    let mut rng = *env.rng;
    let mut ctx = Ctx {
        terrain: &env.terrain,
        rng: &mut rng,
    };
    gather(&mut swarms, &sky, &mut ctx);
    let mut near = Nearest([(f32::INFINITY, Vec3::ZERO, 0.0); LIGHTS]);
    for (mut wander, mut at) in &mut flies {
        let hub = swarms.0[wander.slot % SWARMS];
        let glow = fly(&mut wander, &mut at, &flight, hub, &mut ctx);
        if glow > 0.01 {
            near.offer(
                at.translation.distance_squared(sky.camera),
                at.translation,
                glow,
            );
        }
    }
    light_up(&mut lights, &near, sky.dt);
    *env.rng = rng;
}
