//! Wind: thin speed lines that streak past the body, more of them and longer as it goes faster.

use std::f32::consts::{PI, TAU};

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use super::kit::Kit;
use super::scene::Scene;
use crate::render::smoothstep as smooth;
use crate::screens::Screen;
use crate::space::Rebased;

/// Number of speed lines, created once.
pub const COUNT: u32 = 28;

/// Airspeeds between which the lines appear, in metres per second. None below the first.
pub const SPEED: (f32, f32) = (15.0, 55.0);

/// Length in metres of the tube of air the lines cross, centred on the body.
const DEPTH: f32 = 26.0;

/// Nearest and farthest distance of a line from the flight path through the body, in metres.
const RADIUS: (f32, f32) = (1.3, 4.2);

/// One speed line of the pool.
#[derive(Component, Debug, Clone, Copy)]
pub struct Streak(pub u32);

/// Returns a fixed number in `[0, 1)` for `slot` and `salt`: the same on every run.
pub fn noise(slot: u32, salt: u32) -> f32 {
    let mut x = slot.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x >> 8) as f32 / 16_777_216.0
}

/// Returns how many of the lines show, from 0 to 1, at `airspeed` in air of grip `sigma`.
pub fn strength(airspeed: f32, sigma: f32) -> f32 {
    smooth(SPEED.0, SPEED.1, airspeed) * (sigma * 2.0).min(1.0)
}

/// Spawns the hidden pool when gameplay starts, unless it already exists.
pub fn spawn(mut commands: Commands, kit: Res<Kit>, old: Query<(), With<Streak>>) {
    if !old.is_empty() {
        return;
    }
    for slot in 0..COUNT {
        commands.spawn((
            Streak(slot),
            Mesh3d(kit.quad.clone()),
            MeshMaterial3d(kit.streak.clone()),
            Transform::from_scale(Vec3::ZERO),
            Rebased,
            NotShadowCaster,
            DespawnOnEnter(Screen::Title),
        ));
    }
}

/// Returns the pose of line `slot`, `travel` tube lengths into the flight.
///
/// Each line has its own place around the flight path and runs from ahead of the body to
/// behind it, thin at both ends. It lies along the airflow and turns its face to the camera.
fn pose(slot: u32, travel: f32, scene: &Scene) -> Transform {
    let velocity = scene.velocity();
    let flow = velocity.normalize_or(Vec3::NEG_Z);
    let wing = scene.tilt * scene.state.frame() * Vec3::X;
    let across = (wing - flow * wing.dot(flow)).normalize_or(Vec3::X);
    let turn = noise(slot, 1) * TAU;
    let radius = RADIUS.0 + (RADIUS.1 - RADIUS.0) * noise(slot, 2);
    let passed = (noise(slot, 3) + travel * (0.8 + 0.4 * noise(slot, 4))).fract();
    let ring = (across * turn.cos() + flow.cross(across) * turn.sin()) * radius;
    let at = scene.body + flow * ((0.5 - passed) * DEPTH) + ring;
    let side = flow.cross(scene.eye - at).normalize_or(across);
    let width = (0.012 + 0.012 * noise(slot, 5)) * (PI * passed).sin();
    let length = (velocity.length() * 0.04).clamp(0.6, 4.0);
    Transform {
        translation: at,
        rotation: Quat::from_mat3(&Mat3::from_cols(side, flow, side.cross(flow))),
        scale: Vec3::new(width, length, 1.0),
    }
}

/// Moves the speed lines: the faster the flight, the more of them show.
///
/// `travel` counts the tube lengths flown, so the lines pass at the true airspeed.
pub fn draw(
    scene: Res<Scene>,
    mut travel: Local<f32>,
    mut lines: Query<(&Streak, &mut Transform)>,
) {
    let shown = match scene.active {
        true => strength(scene.state.airspeed, scene.sigma),
        false => 0.0,
    };
    *travel = (*travel + scene.state.airspeed * scene.dt / DEPTH).fract();
    for (streak, mut transform) in &mut lines {
        let visible = (streak.0 as f32 + 0.5) / COUNT as f32 <= shown;
        match visible {
            true => *transform = pose(streak.0, *travel, &scene),
            false => transform.scale = Vec3::ZERO,
        }
    }
}
