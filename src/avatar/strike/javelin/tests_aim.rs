//! Tests of the aim of a throw: the crosshair ray finds the ground it looks at, and a javelin
//! thrown from the hand passes through the point the eyes picked at short range.

use bevy::prelude::*;

use super::super::flight::fly;
use super::aim::{far, launch};
use super::ground::{Ground, Soil};
use super::stage::Slope;
use super::tune::THROW_SIGHT;

/// Looking down at flat ground the ray meets it where the line of sight does; looking up it runs
/// to the end of the sight; an animal on the ray is the point.
#[test]
fn the_crosshair_finds_the_ground() {
    let ground = Slope::rising(0.0, Vec3::NEG_Z, Soil::Hard);
    let eye = Vec3::new(0.0, 1.6, 0.0);
    let down = Vec3::new(0.0, -0.34, -0.94).normalize();
    let met = far(Some(&ground), (eye, down), None);
    let wanted = eye + down * (1.6 / -down.y);
    assert!(met.distance(wanted) < 0.01, "{met} {wanted}");
    assert!(ground.clearance(met) >= 0.0);
    let up = Vec3::new(0.0, 0.5, -0.87).normalize();
    let sky = far(Some(&ground), (eye, up), None);
    assert!(sky.distance(eye + up * THROW_SIGHT) < 1e-3, "{sky}");
    assert!(far(Some(&ground), (eye, up), Some(7.0)).distance(eye + up * 7.0) < 1e-4);
}

/// A javelin thrown from a hand 40 cm to the right of and under the eyes at a point 3 to 6 m ahead
/// passes through it, short only by what gravity takes over that flight.
#[test]
fn a_short_throw_passes_through_the_aimed_point() {
    let (eye, grip) = (Vec3::new(0.0, 1.6, 0.0), Vec3::new(0.4, 1.45, 0.3));
    let g = Vec3::new(0.0, -9.81, 0.0);
    for (distance, pitch) in [(3.0_f32, -0.3_f32), (4.5, 0.0), (6.0, 0.2)] {
        let target = eye + Vec3::new(0.0, pitch.sin(), -pitch.cos()) * distance;
        let (tip, way) = launch(grip, target, 1.1);
        let speed = 24.0;
        let (mut at, mut v, mut nearest) = (tip, way * speed, f32::MAX);
        for _ in 0..600 {
            (at, v) = fly(at, v, g, 0.0, 1.0 / 2400.0);
            nearest = nearest.min(at.distance(target));
        }
        let t = tip.distance(target) / speed;
        let drop = 0.5 * 9.81 * t * t;
        assert!(
            nearest <= drop + 0.02,
            "{distance} m: {nearest} past it, gravity takes {drop}"
        );
    }
}
