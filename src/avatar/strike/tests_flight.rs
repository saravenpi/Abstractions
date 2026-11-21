//! Tests of the thrown spear: ballistic flight, drag, driving into soil and glancing.

use bevy::prelude::*;

use super::flight::{depth, fly, glance};
use super::javelin::tune::{DEEPEST, SKID, SOILS};
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
        (at.x - 20.0).abs() < 0.01 && (at.y - (20.0 - 0.5 * 9.81 * 4.0)).abs() < 0.01,
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

/// A full throw drives the point of a spear into earth deeper than it needs to stand, a slow one
/// does not, and soft snow takes it deeper than earth.
#[test]
fn a_fast_point_drives_in() {
    let held = spear();
    let area = held.build.head.area();
    let fast = 0.5 * held.throw.mass * held.throw.launch * held.throw.launch;
    let [earth, .., snow] = SOILS;
    assert!(depth(fast, area, earth) >= STICK);
    assert!(depth(0.01, area, earth) < STICK);
    assert!(depth(fast, area, snow) > depth(fast, area, earth));
    const { assert!(DEEPEST > STICK) };
}

/// A glance sends back a share of the speed into the surface and keeps a share of the speed along it.
#[test]
fn a_glance_reflects_and_skids() {
    let after = glance(Vec3::new(3.0, -4.0, 0.0), Vec3::Y);
    let wanted = Vec3::new(3.0 * SKID, 4.0 * RESTITUTION, 0.0);
    assert!(after.distance(wanted) < 1e-5, "{after} {wanted}");
}
