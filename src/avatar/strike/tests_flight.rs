//! Tests of the thrown spear: ballistic flight, drag, standing in the ground and bouncing.

use bevy::prelude::*;

use super::flight::{bounce, depth, fly, stands};
use super::stage::spear;
use super::tune::{RESTITUTION, STICK};

/// Without drag the spear follows the parabola of the planet's gravity; with it, the speed falls as
/// e^(−drag·distance).
#[test]
fn a_spear_flies_under_gravity_and_drag() {
    let (g, dt) = (Vec3::new(0.0, -9.81, 0.0), 1.0 / 240.0);
    let (mut at, mut v) = (Vec3::ZERO, Vec3::new(10.0, 10.0, 0.0));
    for _ in 0..480 {
        (at, v) = fly(at, v, g, 0.0, dt);
    }
    assert!(
        (at.x - 20.0).abs() < 0.01 && (at.y - (20.0 - 0.5 * 9.81 * 4.0)).abs() < 0.2,
        "{at}"
    );
    let (mut at, mut v) = (Vec3::ZERO, Vec3::new(30.0, 0.0, 0.0));
    for _ in 0..240 {
        (at, v) = fly(at, v, Vec3::ZERO, 0.01, dt);
    }
    let expected = 30.0 * (-0.01 * at.x).exp();
    assert!(
        (v.length() - expected).abs() / expected < 0.01,
        "{v} {expected}"
    );
}

/// A point arriving fast stands in the ground, a slow or tumbling one does not.
#[test]
fn a_fast_point_stands() {
    let held = spear();
    let area = held.build.head.area();
    let fast = 0.5 * held.throw.mass * held.throw.launch * held.throw.launch;
    assert!(depth(fast, area) >= STICK);
    assert!(stands(true, fast, area));
    assert!(!stands(false, fast, area));
    assert!(!stands(true, 0.01, area));
}

/// A bounce reflects the velocity off the surface and keeps a share of the speed.
#[test]
fn a_bounce_reflects() {
    let after = bounce(Vec3::new(3.0, -4.0, 0.0), Vec3::Y);
    assert!(after.distance(Vec3::new(3.0, 4.0, 0.0) * RESTITUTION) < 1e-5);
}
