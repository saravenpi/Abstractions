//! Tests of the flight of a javelin: without air it follows the parabola of a throw frame by frame,
//! its shaft noses over along the arc, and one balanced behind its middle tumbles instead.

use bevy::prelude::*;

use super::super::flight::{fly, vane};

/// Gravity of the tests, in metres per second squared.
const G: Vec3 = Vec3::new(0.0, -9.81, 0.0);

/// Seconds in a frame at sixty frames per second.
const FRAME: f32 = 1.0 / 60.0;

/// Returns a throw at `speed` metres per second, `pitch` radians above the level, toward -Z.
fn thrown(speed: f32, pitch: f32) -> Vec3 {
    Vec3::new(0.0, pitch.sin(), -pitch.cos()) * speed
}

/// Without drag every frame of the flight lies on the analytic arc x = v·t + ½·g·t², within a
/// millimetre, from a level throw to a steep one.
#[test]
fn the_flight_follows_the_arc() {
    for pitch in [-0.3_f32, 0.0, 0.4, 0.9] {
        let start = thrown(25.0, pitch);
        let (mut at, mut v) = (Vec3::new(0.0, 1.5, 0.0), start);
        for frame in 1..=150 {
            (at, v) = fly(at, v, G, 0.0, FRAME);
            let t = frame as f32 * FRAME;
            let arc = Vec3::new(0.0, 1.5, 0.0) + start * t + 0.5 * G * t * t;
            assert!(
                at.distance(arc) < 1e-3,
                "pitch {pitch}, frame {frame}: {at} {arc}"
            );
        }
    }
}

/// The shaft of a javelin thrown along its path stays within three degrees of its velocity on every
/// frame of the arc, and points down by the time it falls.
#[test]
fn the_shaft_noses_over() {
    let start = thrown(24.0, 0.6);
    let (mut at, mut v, mut axis) = (Vec3::ZERO, start, start.normalize());
    for frame in 0..180 {
        (at, v) = fly(at, v, G, 0.004, FRAME);
        axis = vane(axis, v, true, FRAME);
        let angle = axis.angle_between(v).to_degrees();
        assert!(angle < 3.0, "frame {frame}: {angle}° off the path");
    }
    assert!(v.y < 0.0 && axis.y < -0.3, "{axis} {v} at {at}");
}

/// A javelin balanced behind its middle does not follow its path: its shaft turns away from it.
#[test]
fn a_tail_heavy_javelin_tumbles() {
    let start = thrown(20.0, 0.3);
    let (mut v, mut axis) = (start, start.normalize());
    let mut most: f32 = 0.0;
    for _ in 0..30 {
        (_, v) = fly(Vec3::ZERO, v, G, 0.0, FRAME);
        axis = vane(axis, v, false, FRAME);
        most = most.max(axis.angle_between(v));
    }
    assert!(most > 1.0, "{most}");
}
