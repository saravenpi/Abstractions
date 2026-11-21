//! Tests of walking into hills: a walker turns along a cliff or stops at its foot, never climbs it, and
//! still walks up a slope its feet can hold.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::fixtures::quadruped_with;
use super::{Detail, Motion};
use crate::fauna::react::Mood;
use crate::fauna::steer::{Aim, Pace, drive};
use crate::life::body::{Body, slope};

/// Seconds a walk lasts.
const SECONDS: f32 = 30.0;

/// Seconds per step.
const DT: f32 = 1.0 / 30.0;

/// Walks `body` from the origin towards `goal` over `ground` in `detail`, and returns the root at every
/// step.
fn walk(body: &Body, goal: Vec3, ground: &dyn Fn(Vec2) -> f32, detail: Detail) -> Vec<Vec3> {
    let mut motion = Motion::new(body, Vec3::ZERO, Vec3::X, 7);
    let aim = Aim {
        place: goal,
        flow: Vec3::ZERO,
        excite: 0.0,
    };
    let pace = Pace {
        gravity: 22.0,
        top: 3.0,
        detail,
        strike: 2.0,
        sea: f32::NEG_INFINITY,
        robot: (Vec3::splat(1e4), false),
    };
    let mood = Mood::default();
    (0..(SECONDS / DT) as usize)
        .map(|_| {
            let d = drive(&motion, body, (&aim, &mood), pace, ground);
            motion.step(body, &d, &|p| ground(p), DT);
            motion.root.at
        })
        .collect()
}

/// Returns the ground of a cliff: flat to x = 5 m, then rising 5 m per metre to 20 m.
fn cliff(p: Vec2) -> f32 {
    ((p.x - 5.0) * 5.0).clamp(0.0, 20.0)
}

/// Returns the ground of a ramp rising at `angle` radians from x = 2 m.
fn ramp(p: Vec2, angle: f32) -> f32 {
    (p.x - 2.0).max(0.0) * angle.tan()
}

/// Walks `body` at a cliff over `ground` from a start at `start` metres, straight and slantwise, in
/// both levels of detail, and checks the ground under it never moves more than a tenth of its hip
/// height, and that led slantwise it follows the cliff.
fn held(body: &Body, ground: &dyn Fn(Vec2) -> f32, start: f32) {
    let goals = [Vec3::new(20.0, 0.0, 0.0), Vec3::new(20.0, 0.0, 20.0)];
    for (detail, goal) in [Detail::Full, Detail::Cheap]
        .into_iter()
        .flat_map(|d| goals.map(|g| (d, g)))
    {
        let path = walk(body, goal, ground, detail);
        let worst = path
            .iter()
            .map(|p| (ground(p.xz()) - start).abs())
            .fold(0.0, f32::max);
        let last = path[path.len() - 1];
        println!("{detail:?} to {goal} from {start} m: ground moved {worst:.2} m, ends at {last}");
        assert!(worst <= 0.1 * body.height, "climbed or dropped {worst} m");
        assert!(goal.z == 0.0 || last.z > 10.0, "stuck at {last}");
    }
}

/// A walker led straight at a cliff, or slantwise, from below or from its top, stops at its foot or
/// its edge, and led slantwise it follows the cliff.
#[test]
fn a_cliff_stops_a_walker() {
    let body = quadruped_with(1, 0.4);
    held(&body, &cliff, 0.0);
    held(&body, &|p| 20.0 - cliff(p), 20.0);
}

/// A walker climbs a ramp a little under the steepest slope its feet hold, and the ground under it
/// rises no faster than its speed allows on that slope.
#[test]
fn a_walker_climbs_a_slope_it_can_hold() {
    let body = quadruped_with(1, 0.4);
    let angle = 0.9 * slope(&body);
    let ground = |p: Vec2| ramp(p, angle);
    let path = walk(&body, Vec3::new(30.0, 0.0, 0.0), &ground, Detail::Full);
    let last = path[path.len() - 1];
    assert!(last.x > 20.0, "stopped at {last}");
    let fastest = path
        .windows(2)
        .map(|w| (ground(w[1].xz()) - ground(w[0].xz())) / DT)
        .fold(0.0, f32::max);
    assert!(fastest <= 3.0 * angle.sin() * 1.5, "rose {fastest} m/s");
}
