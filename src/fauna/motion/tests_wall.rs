//! Tests of the flight around things: a flyer drifting at a wall rises or slows before it, and its body
//! is never inside the wall on any frame.

use bevy::math::{Vec2, Vec3};

use super::fixtures_soft::flyer;
use super::rig::{Detail, Drive, Motion};
use crate::fauna::bulk::capsule;
use crate::fauna::wing::{clamp, look};

/// Seconds per step.
const DT: f32 = 1.0 / 30.0;

/// Where the face of the test wall stands along x, in metres, and how high it rises.
const WALL: (f32, f32) = (10.0, 30.0);

/// Returns how far a ray from `from` toward `to` meets the wall: a slab from x = [`WALL`] on, from
/// the ground to its top.
fn wall(from: Vec3, to: Vec3) -> Option<f32> {
    let way = to - from;
    if way.x <= 0.0 || to.x < WALL.0 {
        return None;
    }
    let t = ((WALL.0 - from.x) / way.x).max(0.0);
    let at = from + way * t;
    (at.y <= WALL.1).then(|| t * way.length())
}

/// A flyer driven straight at a wall at 4 m/s, 2 m over flat ground, for eight seconds, slows or rises
/// before it and never has the middle of its body within half its radius of the wall on any frame: a
/// step that went into the wall is taken back to a radius short of it along the way it went.
#[test]
fn a_flyer_never_flies_into_a_wall() {
    let body = flyer();
    let (radius, _) = capsule(body.length, body.height);
    let mut m = Motion::new(&body, Vec3::new(0.0, 2.0, 0.0), Vec3::X, 29);
    let drive = Drive {
        velocity: Vec3::new(4.0, 0.0, 0.0),
        look: None,
        gravity: 9.81,
        excite: 0.0,
        detail: Detail::Full,
    };
    let (start, mut highest) = (m.root.at.y, m.root.at.y);
    for frame in 0..240 {
        m.step(&body, &drive, &|_: Vec2| 0.0, DT);
        clamp(&mut m, &wall, (radius, DT));
        look(&mut m, &wall, (radius, DT));
        let inside = m.root.at.x + 0.5 * radius - WALL.0;
        assert!(
            inside <= 1e-3,
            "frame {frame}: the body is {inside} m inside the wall"
        );
        highest = highest.max(m.root.at.y);
    }
    let rose = highest - start;
    let stood = WALL.0 - m.root.at.x;
    assert!(
        rose > 0.5 || stood > radius,
        "it neither rose ({rose} m) nor held off ({stood} m)"
    );
}
