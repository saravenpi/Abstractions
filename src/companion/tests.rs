//! Tests of the spring and the resting spot.

use std::sync::Arc;

use bevy::prelude::*;

use super::spot::{Probe, Stance};
use super::spring::{rate, step, stiffness};
use super::tune::{CLEARANCE, FOLLOW_RATE, YIELD};
use crate::space::Anchor;
use crate::universe::{BodyId, EARTH_RADIUS, STANDARD_GRAVITY};
use crate::world::globe::Face;
use crate::world::{Globe, Terrain};

/// Returns the eight compass directions and two steep views, as unit vectors.
fn views() -> Vec<Vec3> {
    let flat = (0..8).map(|i| Quat::from_rotation_y(i as f32 * 0.785) * Vec3::NEG_Z);
    let steep = [Vec3::new(0.02, -1.0, 0.01), Vec3::new(0.3, 0.9, -0.2)];
    flat.chain(steep).map(Vec3::normalize).collect()
}

/// Returns the twenty steepest land points of a coarse scan of `terrain`, as chart positions on the ground.
fn steepest(terrain: &Terrain, span: i32) -> Vec<Vec3> {
    let slope = |x: f32, z: f32| {
        let (dx, dz) = (
            terrain.height(x + 1.0, z) - terrain.height(x - 1.0, z),
            terrain.height(x, z + 1.0) - terrain.height(x, z - 1.0),
        );
        dx.hypot(dz) * 0.5
    };
    let mut found: Vec<(f32, Vec3)> = (-span..span)
        .flat_map(|i| (-span..span).map(move |j| (i as f32 * 23.0, j as f32 * 23.0)))
        .map(|(x, z)| (slope(x, z), Vec3::new(x, terrain.height(x, z), z)))
        .filter(|(_, at)| at.y > 0.5)
        .collect();
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    found.into_iter().take(20).map(|(_, at)| at).collect()
}

/// Checks that every resting and reappearing spot around `ground` clears the ground and the water.
fn assert_clear(probe: &Probe, ground: Vec3) {
    let up = probe.up(ground);
    for (forward, lift) in views().into_iter().flat_map(|v| [(v, 1.6), (v, -6.0)]) {
        let stance = Stance {
            eye: ground + up * lift,
            forward,
            up,
        };
        for spot in [stance.rest(stance.right()), stance.behind()] {
            let gap = probe.gap(probe.clear(spot, CLEARANCE));
            assert!(gap >= CLEARANCE - 2e-3, "gap {gap} at {ground}");
        }
    }
}

/// A spring released at rest reaches its target within two seconds and never passes it, at any frame rate.
#[test]
fn spring_converges_without_overshoot() {
    for dt in [1.0 / 20.0, 1.0 / 60.0, 1.0 / 144.0] {
        let start = Vec3::new(3.0, 1.0, -2.0);
        let (mut offset, mut vel) = (start, Vec3::ZERO);
        for _ in 0..(2.0 / dt) as usize {
            (offset, vel) = step(offset, vel, FOLLOW_RATE, dt);
            let along = offset.dot(start.normalize());
            assert!(along > -1e-3 * start.length(), "overshoot {along}");
        }
        assert!(offset.length() < 0.01 * start.length(), "left {offset}");
        assert!(vel.length() < 0.05);
    }
}

/// The stiffening spring used for catching up closes thirty meters in two seconds without flying past.
#[test]
fn stiff_spring_catches_up() {
    let dt = 1.0 / 60.0;
    let (mut offset, mut vel) = (Vec3::X * 30.0, Vec3::ZERO);
    for _ in 0..120 {
        (offset, vel) = step(offset, vel, rate(offset, vel), dt);
        assert!(offset.x > -0.01, "overshoot {}", offset.x);
    }
    assert!(offset.length() < 0.1, "left {offset}");
    assert!(stiffness(0.0) < stiffness(3.0) && stiffness(3.0) < stiffness(50.0));
}

/// On the flat test world the spot clears the ground and the sea, on the steepest slopes too.
#[test]
fn spot_clears_steep_ground_and_water() {
    for seed in [1, 42] {
        let terrain = Terrain::new(seed);
        let anchor = Anchor::surface(BodyId::default(), &terrain, Vec3::ZERO);
        let probe = Probe {
            anchor: &anchor,
            terrain: &terrain,
        };
        let steep = steepest(&terrain, 60);
        assert_eq!(steep.len(), 20);
        for ground in steep.into_iter().chain([Vec3::new(5000.0, -30.0, 5000.0)]) {
            assert_clear(&probe, ground);
        }
    }
}

/// On a sphere planet the spot clears the ground and the sea along the local vertical.
#[test]
fn spot_clears_ground_on_a_sphere() {
    let globe = Arc::new(Globe::new(7, EARTH_RADIUS, STANDARD_GRAVITY));
    let terrain = Terrain::on(globe.clone(), globe.face_chart(Face::PosY));
    for at in steepest(&terrain, 30).into_iter().take(6) {
        let anchor = Anchor::surface(BodyId::default(), &terrain, at);
        let probe = Probe {
            anchor: &anchor,
            terrain: &terrain,
        };
        assert!(probe.gap(at).abs() < 0.05, "gap {}", probe.gap(at));
        assert_clear(&probe, at);
        assert_clear(&probe, at + Vec3::new(40.0, 0.0, -25.0));
    }
}

/// A spot on the line of sight is pushed out to the edge of the clear cone, a spot already out
/// of it or behind the eyes stays.
#[test]
fn yields_from_the_line_of_sight() {
    let stance = Stance {
        eye: Vec3::new(3.0, 2.0, 1.0),
        forward: Vec3::NEG_Z,
        up: Vec3::Y,
    };
    let ahead = stance.eye + Vec3::new(0.1, 0.0, -4.0);
    let moved = stance.aside(ahead) - stance.eye;
    assert!((moved.z + 4.0).abs() < 1e-4 && (moved.x - 4.0 * YIELD).abs() < 1e-4);
    for kept in [Vec3::new(3.0, 0.0, -2.0), Vec3::new(0.0, 0.0, 1.0)] {
        assert_eq!(stance.aside(stance.eye + kept), stance.eye + kept);
    }
}
