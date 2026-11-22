//! Tests for jumping against steep/walkable hill slopes and descending curved trimesh slopes.

use avian3d::prelude::*;
use bevy::prelude::*;

use super::stage::{app, body};
use crate::player::asks::gather;
use crate::player::controller::{Motion, drive};
use crate::player::intent::JUMP_SPEED;
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::sim::{Intent, IntentOf, PlayerId};

/// Height of the stage flat floor and distance from body centre to soles.
const FLOOR: f32 = 10.5;

/// Distance from body centre down to its soles.
const SOLES: f32 = BODY_LENGTH * 0.5 + BODY_RADIUS;

/// Spawns a sloping box rising ahead toward -Z starting at `(z = -2.0, y = FLOOR)`.
fn spawn_ramp(world: &mut World, angle_deg: f32) {
    let q = Quat::from_rotation_x(angle_deg.to_radians());
    let centre = Vec3::new(0.0, FLOOR, -2.0) - q * Vec3::new(0.0, 0.5, 10.0);
    world.spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_translation(centre).with_rotation(q),
    ));
}

/// Runs `frames` ticks pushing `intents` and returns the body position and velocity on each frame.
fn run_frames(world_app: &mut App, intents: &[Intent], frames: usize) -> Vec<(Vec3, Vec3, bool)> {
    let mut out = Vec::with_capacity(frames);
    for _ in 0..frames {
        for &intent in intents {
            world_app.world_mut().write_message(IntentOf {
                player: PlayerId::LOCAL,
                intent,
            });
        }
        world_app.update();
        let (transform, motion, _) = body(world_app, PlayerId::LOCAL);
        out.push((transform.translation, motion.velocity, motion.grounded));
    }
    out
}

/// Builds a grid trimesh collider strip from `z_from` down to `z_to` using `height(z)`.
fn hill_trimesh(z_from: f32, z_to: f32, height: impl Fn(f32) -> f32) -> Collider {
    let cols = 5usize;
    let rows = 9usize;
    let mut verts = Vec::with_capacity(cols * rows);
    let mut tris = Vec::with_capacity((cols - 1) * (rows - 1) * 2);
    for r in 0..rows {
        let z = z_from + (z_to - z_from) * (r as f32 / (rows - 1) as f32);
        let y = height(z);
        for c in 0..cols {
            let x = -8.0 + 4.0 * c as f32;
            verts.push(Vec3::new(x, y, z));
        }
    }
    for r in 0..(rows - 1) {
        for c in 0..(cols - 1) {
            let a = (r * cols + c) as u32;
            let b = ((r + 1) * cols + c) as u32;
            tris.push([a, a + 1, b]);
            tris.push([b, a + 1, b + 1]);
        }
    }
    Collider::trimesh_with_config(verts, tris, TrimeshFlags::FIX_INTERNAL_EDGES)
}

/// Jumping and sprinting directly into a steep 60-degree hillside never climbs or launches upward.
#[test]
fn jumping_into_a_steep_slope_never_climbs_or_flies() {
    let mut world_app = app(&[(PlayerId::LOCAL, 0.0)]);
    world_app.add_systems(Update, (gather, drive).chain());
    spawn_ramp(world_app.world_mut(), 60.0);
    run_frames(&mut world_app, &[], 5);
    let move_jump = [
        Intent::Move {
            forward: 1.0,
            strafe: 0.0,
        },
        Intent::Boost,
        Intent::Jump,
    ];
    let log = run_frames(&mut world_app, &move_jump, 45);
    let max_y = log.iter().map(|s| s.0.y).fold(f32::NEG_INFINITY, f32::max);
    let min_z = log.iter().map(|s| s.0.z).fold(f32::INFINITY, f32::min);
    let peak_vy = log.iter().map(|s| s.1.y).fold(f32::NEG_INFINITY, f32::max);
    assert!(max_y - (FLOOR + SOLES) < 1.35, "climbed to {max_y}");
    assert!(min_z > -2.8, "climbed past cliff base to z={min_z}");
    assert!(peak_vy <= JUMP_SPEED + 0.05, "launched at vy={peak_vy}");
}

/// Jumping into a walkable 42-degree slope never converts forward sprint speed into extra vertical lift.
#[test]
fn jumping_into_a_walkable_hill_never_boosts_jump_height() {
    let mut world_app = app(&[(PlayerId::LOCAL, 0.0)]);
    world_app.add_systems(Update, (gather, drive).chain());
    spawn_ramp(world_app.world_mut(), 42.0);
    run_frames(&mut world_app, &[], 5);
    let jump_sprint = [
        Intent::Move {
            forward: 1.0,
            strafe: 0.0,
        },
        Intent::Boost,
        Intent::Jump,
    ];
    let log = run_frames(&mut world_app, &jump_sprint, 20);
    let max_air_vy = log
        .iter()
        .filter(|s| !s.2)
        .map(|s| s.1.y)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_air_vy <= JUMP_SPEED + 0.05,
        "airborne vy spiked to {max_air_vy}"
    );
}

/// Walking and sprinting downhill across curved trimesh terrain and a chunk seam never gets blocked.
#[test]
fn descending_a_trimesh_hill_never_blocks() {
    let hill_y = |z: f32| {
        if z > 4.0 {
            16.0 + 0.18 * (z - 4.0)
        } else if z > -8.0 {
            16.0 - 0.65 * (4.0 - z)
        } else {
            16.0 - 0.65 * 12.0 - 0.05 * (-8.0 - z)
        }
    };
    let mut world_app = app(&[(PlayerId::LOCAL, 0.0)]);
    world_app.add_systems(Update, (gather, drive).chain());
    {
        let world = world_app.world_mut();
        world.spawn((
            RigidBody::Static,
            hill_trimesh(12.0, 0.0, hill_y),
            Transform::default(),
        ));
        world.spawn((
            RigidBody::Static,
            hill_trimesh(0.0, -16.0, hill_y),
            Transform::default(),
        ));
        let mut q = world.query_filtered::<(&mut Transform, &mut Motion), With<Player>>();
        for (mut t, mut m) in q.iter_mut(world) {
            t.translation = Vec3::new(0.0, hill_y(10.0) + SOLES + 0.03, 10.0);
            *m = Motion::default();
        }
    }
    run_frames(&mut world_app, &[], 5);
    let downhill = [
        Intent::Move {
            forward: 1.0,
            strafe: 0.0,
        },
        Intent::Boost,
    ];
    let log = run_frames(&mut world_app, &downhill, 120);
    let end_z = log.last().expect("frames").0.z;
    assert!(end_z < -10.0, "blocked on downhill trimesh at z={end_z}");
    assert!(
        log.iter().all(|s| s.0.z < 10.5 && s.2),
        "ungrounded downhill"
    );
}
