//! The puffs the body kicks up: a fixed pool of small quads thrown by the rules of `dust`.
//!
//! The entities, the mesh and the materials are made once; a burst only wakes parked puffs,
//! so nothing is spawned or allocated per event. A puff is free in render space and marked
//! `Rebased`, so an anchor rebase carries it. It swells, then shrinks to nothing: it never pops
//! out. It is an ordinary depth-tested mesh, lit like the ground it came from.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::dust::{Burst, From, Soil, Way};
use super::marks::Marks;
use crate::render::smoothstep as smooth;
use crate::screens::Screen;
use crate::space::{Anchor, Rebased};
use crate::universe::PlanetClass;
use crate::world::Terrain;

/// Number of puffs, created once: a hard landing takes 18, a run two or three at a time.
pub const COUNT: usize = 56;

/// Drag of home air on a puff, per second. Thinner air drags less, and none not at all.
const DRAG: f32 = 3.2;

/// Share of its weight a puff keeps in home air, where fine dust all but hangs. With no air
/// it keeps all of it and flies as a thrown stone does.
const HANG: f32 = 0.12;

/// One pooled puff, free in render space.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Puff {
    /// Velocity in render axes, in meters per second.
    pub velocity: Vec3,
    /// Seconds since it was thrown.
    pub age: f32,
    /// Seconds it lasts. Zero while parked.
    pub life: f32,
    /// Its largest size in meters.
    pub size: f32,
}

/// The materials the puffs wear.
#[derive(Resource, Debug, Clone)]
pub struct Puffs {
    /// One material per soil, in the order of [`Soil::ALL`].
    pub soils: [Handle<StandardMaterial>; Soil::ALL.len()],
    /// The class of planet the materials are coloured for.
    pub class: Option<PlanetClass>,
}

/// Returns the lit, see-through, two-sided material of a puff of `tint`, an sRGB colour with alpha.
pub fn material([r, g, b, a]: [f32; 4]) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(r, g, b, a),
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// Builds the pool when gameplay starts, unless it already exists: the shared mesh, one
/// material per soil, and the parked puffs.
pub fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    old: Query<(), With<Puff>>,
) {
    if !old.is_empty() {
        return;
    }
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let soils = Soil::ALL.map(|soil| materials.add(material(soil.tint(PlanetClass::Island))));
    for _ in 0..COUNT {
        commands.spawn((
            Puff::default(),
            Mesh3d(quad.clone()),
            MeshMaterial3d(soils[0].clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
    commands.insert_resource(Puffs {
        soils,
        class: Some(PlanetClass::Island),
    });
}

/// Returns a fixed number in `[0, 1)` for `serial` and `salt`: the same on every run.
pub fn noise(serial: u32, salt: u32) -> f32 {
    let mut x = serial.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x >> 8) as f32 / 16_777_216.0
}

/// Where the body is and how it moves, in render space, for the puffs it throws.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Thrower {
    /// The pose of the player entity.
    pub pose: Transform,
    /// Its velocity in meters per second.
    pub velocity: Vec3,
    /// The local vertical.
    pub up: Vec3,
}

/// Returns puff number `serial` of `burst`, newly thrown by `thrower` whose gear sits at
/// `marks`, and where it starts.
pub fn throw(burst: &Burst, serial: u32, thrower: &Thrower, marks: &Marks) -> (Puff, Vec3) {
    let (up, pose) = (thrower.up, &thrower.pose);
    let flat = thrower.velocity - up * thrower.velocity.dot(up);
    let ahead = flat.normalize_or(pose.forward().as_vec3());
    let turn = Quat::from_axis_angle(up, noise(serial, 1) * TAU);
    let around = turn * up.any_orthonormal_vector();
    let side = (serial % 2) as usize;
    let from = match burst.from {
        From::Feet => (marks.soles[0] + marks.soles[1]) * 0.5,
        From::Foot(foot) => marks.soles[foot],
        From::Tips => marks.tips[side],
    };
    let fast = burst.speed * (0.6 + 0.8 * noise(serial, 2));
    let lift = up * (0.25 * fast * noise(serial, 3));
    let velocity = match burst.way {
        Way::Ring => around * fast + lift,
        Way::Back => (around * 0.4 - ahead).normalize() * fast + lift,
        Way::Ahead => (around * 0.4 + ahead).normalize() * fast + lift + flat * 0.3,
        Way::Wake => thrower.velocity * 0.12 + around * fast,
    };
    let puff = Puff {
        velocity,
        age: 0.0,
        life: burst.life * (0.7 + 0.6 * noise(serial, 4)),
        size: burst.size * (0.7 + 0.6 * noise(serial, 5)),
    };
    (puff, pose.transform_point(from) + around * 0.12)
}

/// Advances a live puff by `dt` seconds under a pull of `weight` meters per second squared
/// along `up`, in air `air` times as thick as at home, and returns its size now.
///
/// In air it slows down and hangs; with no air nothing slows it and it flies its whole arc.
/// It grows out of nothing, swells as it thins and shrinks to nothing over the end of its life.
pub fn drift(puff: &mut Puff, at: &mut Vec3, (up, weight, air): (Vec3, f32, f32), dt: f32) -> f32 {
    let air = air.clamp(0.0, 1.0);
    puff.age += dt;
    puff.velocity -= up * (weight * (1.0 - (1.0 - HANG) * air) * dt);
    puff.velocity *= (-DRAG * air * dt).exp();
    *at += puff.velocity * dt;
    let spent = (puff.age / puff.life).min(1.0);
    puff.size * smooth(0.0, 0.2, spent) * (0.5 + 0.5 * spent) * (1.0 - smooth(0.55, 1.0, spent))
}

/// Run condition: the pool exists and so does a world to throw puffs in.
pub fn ready(
    puffs: Option<Res<Puffs>>,
    anchor: Option<Res<Anchor>>,
    terrain: Option<Res<Terrain>>,
) -> bool {
    puffs.is_some() && anchor.is_some() && terrain.is_some()
}

/// Colours the materials of the pool for the planet of `class`, once per planet.
pub fn recolour(puffs: &mut Puffs, class: PlanetClass, materials: &mut Assets<StandardMaterial>) {
    if puffs.class == Some(class) {
        return;
    }
    puffs.class = Some(class);
    for (handle, soil) in puffs.soils.iter().zip(Soil::ALL) {
        if let Some(mut paint) = materials.get_mut(handle) {
            paint.base_color = material(soil.tint(class)).base_color;
        }
    }
}
