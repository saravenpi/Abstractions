//! Small pooled particles of the boots: sparks while they brake a fall, and a ring of dust or
//! spray when they push close over ground or water.

use std::f32::consts::TAU;

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::kit::Kit;
use super::scene::{SOLES, Scene};
use super::streaks::noise;
use crate::gear::Mode;
use crate::render::smoothstep as smooth;
use crate::screens::Screen;
use crate::space::Rebased;

/// Number of particles, created once.
pub const COUNT: usize = 40;

/// Sparks per second at the hardest braking.
const SPARK_RATE: f32 = 45.0;

/// Puffs per second at full push right over the surface.
const PUFF_RATE: f32 = 26.0;

/// Free height under the soles below which the boots raise dust or spray, in metres.
pub const PUFF_HEIGHT: f32 = 3.5;

/// What a particle is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sort {
    /// Parked and invisible.
    #[default]
    Idle,
    /// A hot spark falling from a nozzle.
    Spark,
    /// A puff of dust rolling out over the ground.
    Dust,
    /// A puff of spray rolling out over water.
    Spray,
}

/// One pooled particle, free in render space.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Mote {
    /// What it is right now.
    pub sort: Sort,
    /// Velocity in render axes, in metres per second.
    pub velocity: Vec3,
    /// Seconds since it was emitted.
    pub age: f32,
    /// Seconds it lives for.
    pub life: f32,
}

/// How many particles are owed, carried between frames, and a counter that seeds them.
#[derive(Debug, Clone, Copy, Default)]
pub struct Owed {
    /// Sparks owed.
    sparks: f32,
    /// Puffs owed.
    puffs: f32,
    /// Particles emitted so far.
    serial: u32,
}

/// Spawns the hidden pool when gameplay starts, unless it already exists.
pub fn spawn(mut commands: Commands, kit: Res<Kit>, old: Query<(), With<Mote>>) {
    if !old.is_empty() {
        return;
    }
    for _ in 0..COUNT {
        commands.spawn((
            Mote::default(),
            Mesh3d(kit.quad.clone()),
            MeshMaterial3d(kit.spark.clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Returns the sparks and puffs per second the boots throw in this scene.
///
/// Sparks need the boots to hold more than the weight, which is a braked fall. Puffs need a
/// surface close under the soles.
pub fn rates(scene: &Scene) -> (f32, f32) {
    let push = scene.push();
    let braking = scene.state.mode == Mode::BootFall;
    let hard = smooth(1.15, 2.2, scene.state.load) * f32::from(u8::from(braking));
    let near = 1.0 - smooth(0.5, PUFF_HEIGHT, scene.clearance);
    (SPARK_RATE * push * hard, PUFF_RATE * push * near)
}

/// Returns particle number `serial` of `sort`, newly emitted, and where it starts.
fn emit(sort: Sort, serial: u32, scene: &Scene) -> (Mote, Vec3) {
    let up = scene.tilt * Vec3::Y;
    let turn = noise(serial, 11) * TAU;
    let out = scene.tilt * Vec3::new(turn.cos(), 0.0, turn.sin());
    let soles = scene.at(Vec3::NEG_Y * SOLES);
    let (velocity, at, life) = match sort {
        Sort::Spark => (
            scene.velocity() * 0.4 + out * (1.0 + 2.5 * noise(serial, 12))
                - up * (3.0 + 5.0 * noise(serial, 13)),
            soles,
            0.25 + 0.3 * noise(serial, 14),
        ),
        _ => (
            out * (3.0 + 3.5 * noise(serial, 12)) + up * (0.3 + 0.8 * noise(serial, 13)),
            soles - up * scene.clearance + out * 0.25,
            0.5 + 0.4 * noise(serial, 14),
        ),
    };
    (
        Mote {
            sort,
            velocity,
            age: 0.0,
            life,
        },
        at,
    )
}

/// Advances one live particle by the frame of `scene` and returns its scale.
///
/// A spark falls and shrinks. A puff slows down, swells, then thins away.
fn drift(mote: &mut Mote, at: &mut Vec3, scene: &Scene) -> f32 {
    mote.age += scene.dt;
    *at += mote.velocity * scene.dt;
    let spent = (mote.age / mote.life).min(1.0);
    match mote.sort {
        Sort::Spark => {
            mote.velocity -= scene.tilt * Vec3::Y * (14.0 * scene.dt);
            0.035 * (1.0 - spent * spent)
        }
        _ => {
            mote.velocity *= (-3.0 * scene.dt).exp();
            (0.12 + 0.5 * spent) * (1.0 - smooth(0.6, 1.0, spent))
        }
    }
}

/// Returns the sort of the next particle to emit, taking it off what is owed, if any is owed.
fn due(owed: &mut Owed, wet: bool) -> Option<Sort> {
    let sort = if owed.sparks >= 1.0 {
        owed.sparks -= 1.0;
        Sort::Spark
    } else if owed.puffs >= 1.0 {
        owed.puffs -= 1.0;
        if wet { Sort::Spray } else { Sort::Dust }
    } else {
        return None;
    };
    owed.serial = owed.serial.wrapping_add(1);
    Some(sort)
}

/// Returns the material particles of `sort` wear.
fn wear(kit: &Kit, sort: Sort) -> MeshMaterial3d<StandardMaterial> {
    MeshMaterial3d(match sort {
        Sort::Dust => kit.dust.clone(),
        Sort::Spray => kit.spray.clone(),
        _ => kit.spark.clone(),
    })
}

/// Moves the live particles, parks the spent ones and emits what the boots owe into parked ones.
pub fn step(
    scene: Res<Scene>,
    kit: Res<Kit>,
    mut owed: Local<Owed>,
    mut motes: Query<(
        &mut Mote,
        &mut Transform,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let (sparks, puffs) = rates(&scene);
    owed.sparks = (owed.sparks + sparks * scene.dt).min(8.0);
    owed.puffs = (owed.puffs + puffs * scene.dt).min(8.0);
    for (mut mote, mut transform, mut material) in &mut motes {
        if mote.sort != Sort::Idle && mote.age < mote.life {
            let size = drift(&mut mote, &mut transform.translation, &scene);
            transform.scale = Vec3::splat(size);
            transform.rotation = scene.view;
        } else if let Some(sort) = due(&mut owed, scene.wet) {
            (*mote, transform.translation) = emit(sort, owed.serial, &scene);
            material.set_if_neq(wear(&kit, sort));
        } else if transform.scale != Vec3::ZERO {
            mote.sort = Sort::Idle;
            transform.scale = Vec3::ZERO;
        }
    }
}
