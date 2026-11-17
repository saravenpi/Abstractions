//! Night fireflies: glowing bodies wandering in loose swarms, each of the nearest few carrying its own faint light.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::anim::blink;
use super::env::{Ctx, Env, Sky};
use super::gear::{Bind, Gear, rebind};
use super::habitat::{Species, presence};
use super::steer::{approach, step_toward};
use super::wander::{PARKED, Roam, Wander, dormant, pick, place, strayed};
use crate::screens::Screen;
use crate::space::{Rebased, Shift};

/// Fireflies in the pool.
const COUNT: usize = 24;

/// Fireflies that carry a real light, at most.
pub const LIGHTS: usize = 6;

/// Loose swarms the fireflies gather around.
const SWARMS: usize = 4;

/// Body diameter in meters.
const SIZE: f32 = 0.09;

/// Light power in lumens at full glow: a faint splash on the ground, the glowing body carries the look.
const LIGHT_POWER: f32 = 300.0;

/// Reach of a firefly light in meters.
const LIGHT_RANGE: f32 = 3.0;

/// Fireflies farther than this from the camera carry no light, in meters. It is inside the leash on purpose.
const LIGHT_REACH: f32 = 22.0;

/// How fast a light fades when it takes or leaves a firefly, in levels per second.
const FADE: f32 = 3.0;

/// How fireflies roam around their swarm.
pub(super) const ROAM: Roam = Roam {
    radius: 6.0,
    reach: 4.0,
    leash: 28.0,
    band: (0.3, 2.5),
    pause: (1.0, 4.0),
    land: 0.0,
    speed: 0.5,
};

/// Transform, light, visibility and binding of a firefly light.
type LightData = (
    &'static mut Transform,
    &'static mut PointLight,
    &'static mut Visibility,
    &'static mut Bind,
);

/// Marks a firefly body.
#[derive(Component, Debug, Clone, Copy)]
pub struct Firefly;

/// Anchor points of the swarms, in render space. Only their place along the ground matters.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Swarms([Vec3; SWARMS]);

impl Swarms {
    /// Moves the anchor points with render space after an anchor rebase.
    pub fn carry(&mut self, shift: &Shift) {
        for anchor in &mut self.0 {
            *anchor = shift.point(*anchor);
        }
    }
}

/// What the lights need to know about one firefly this frame.
#[derive(Debug, Clone, Copy, Default)]
struct Seen {
    /// Where it is.
    at: Vec3,
    /// How brightly it shines, from 0 to 1.
    glow: f32,
    /// Distance to the camera while it is out and close enough to carry a light.
    dist: Option<f32>,
}

/// What a firefly update needs to know about the sky.
#[derive(Debug, Clone, Copy)]
pub(super) struct Flight {
    /// The frame's view of the world.
    pub sky: Sky,
    /// Number of fireflies that should be out.
    pub level: f32,
}

/// Spawns the fireflies and their lights. Fireflies are free entities in render space, so each
/// is marked to follow anchor rebases. A light is put on its firefly every frame it shines, so
/// it needs no mark.
pub fn spawn(commands: &mut Commands, gear: &Gear) {
    for slot in 0..COUNT {
        commands.spawn((
            Firefly,
            Wander::new(slot),
            Mesh3d(gear.glow_mesh.clone()),
            MeshMaterial3d(gear.glow.clone()),
            Transform::from_translation(PARKED).with_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
    for _ in 0..LIGHTS {
        commands.spawn((
            Bind::default(),
            PointLight {
                color: Color::srgb(0.8, 1.0, 0.35),
                intensity: 0.0,
                range: LIGHT_RANGE,
                radius: 0.12,
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
        if strayed(*anchor, ctx, &ROAM) {
            let angle = ctx.rng.range(0.0, TAU);
            let reach = ctx.rng.range(6.0, 20.0);
            *anchor =
                sky.camera + sky.tilt * Vec3::new(angle.cos() * reach, 0.0, angle.sin() * reach);
        }
    }
}

/// Advances one firefly and returns how brightly it shines.
pub(super) fn fly(
    wander: &mut Wander,
    at: &mut Transform,
    flight: &Flight,
    hub: Vec3,
    ctx: &mut Ctx,
) -> f32 {
    let Flight { sky, level } = *flight;
    let dt = sky.dt;
    wander.shown = approach(
        wander.shown,
        (level - wander.slot as f32).clamp(0.0, 1.0),
        dt * 0.4,
    );
    wander.phase = (wander.phase + dt * (1.1 + 0.1 * (wander.slot % 7) as f32)) % TAU;
    if strayed(at.translation, ctx, &ROAM) {
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
    let hover = Vec3::new((p * 0.7).sin(), 0.5 * (p * 1.3).cos(), (p * 0.9).cos());
    at.translation += sky.tilt * hover * 0.15 * dt;
    let glow = blink(p, sky.ambient.daylight) * wander.shown;
    at.scale = Vec3::splat(SIZE * wander.shown * (0.15 + 0.85 * glow));
    glow
}

/// Keeps every light on its own firefly: it sits exactly on it and shines with its blink and its own fade.
fn light_up(lights: &mut Query<LightData, Without<Firefly>>, seen: &[Seen; COUNT], dt: f32) {
    let mut binds = [Bind::default(); LIGHTS];
    for (slot, (.., bind)) in binds.iter_mut().zip(lights.iter()) {
        *slot = *bind;
    }
    let dists = seen.map(|firefly| firefly.dist);
    rebind(&mut binds, &dists, FADE * dt);
    for ((mut at, mut light, mut visibility, mut bind), next) in lights.iter_mut().zip(binds) {
        bind.set_if_neq(next);
        let carried = next.target.and_then(|slot| seen.get(slot));
        if let Some(firefly) = carried {
            at.translation = firefly.at;
        }
        let power = LIGHT_POWER * next.level * carried.map_or(0.0, |firefly| firefly.glow);
        light.intensity = power;
        visibility.set_if_neq(match power > 0.0 {
            true => Visibility::Visible,
            false => Visibility::Hidden,
        });
    }
}

/// Moves the swarms, the fireflies that are out or wanted, and their lights. The other
/// fireflies rest untouched.
pub fn step(
    mut env: Env,
    mut swarms: ResMut<Swarms>,
    mut flies: Query<(&mut Wander, &mut Transform), With<Firefly>>,
    mut lights: Query<LightData, Without<Firefly>>,
) {
    let sky = env.sky();
    let level = presence(Species::Firefly, &sky.ambient) * COUNT as f32;
    let flight = Flight { sky, level };
    let mut rng = *env.rng;
    let mut ctx = Ctx {
        ground: env.ground(),
        rng: &mut rng,
    };
    gather(&mut swarms, &sky, &mut ctx);
    let mut seen = [Seen::default(); COUNT];
    for (mut wander, mut at) in &mut flies {
        if dormant(&wander, level) {
            continue;
        }
        let hub = swarms.0[wander.slot % SWARMS];
        let glow = fly(&mut wander, &mut at, &flight, hub, &mut ctx);
        let dist = at.translation.distance(sky.camera);
        if let Some(firefly) = seen.get_mut(wander.slot) {
            *firefly = Seen {
                at: at.translation,
                glow,
                dist: (wander.shown > 0.05 && dist < LIGHT_REACH).then_some(dist),
            };
        }
    }
    light_up(&mut lights, &seen, sky.dt);
    *env.rng = rng;
}
