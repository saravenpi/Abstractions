//! Tests of a pursuit: an animal that goes for the robot closes on a robot standing still, from far,
//! at its full speed once near, and stops where its attack reaches.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::fixtures::quadruped_with;
use super::{Detail, Motion};
use crate::fauna::react::Mood;
use crate::fauna::senses::Stance;
use crate::fauna::steer::{Aim, Pace, drive};

/// Seconds per step.
const DT: f32 = 1.0 / 30.0;

/// Metres from its root within which the attack of the test animal reaches.
const STRIKE: f32 = 1.5;

/// Returns the pace of an animal of 8 m/s top speed going for a robot standing at `robot` on flat dry
/// ground.
fn pace(robot: Vec3) -> Pace {
    Pace {
        gravity: 9.81,
        top: 8.0,
        detail: Detail::Full,
        strike: STRIKE,
        sea: f32::NEG_INFINITY,
        robot: (robot, false),
    }
}

/// An animal going for a robot that stands still 25 m away closes on it within twelve seconds, never
/// runs past it, and stands within half a metre past its strike once there: it keeps hunting a robot
/// that stopped moving.
#[test]
fn a_hunter_closes_on_a_robot_that_stands_still() {
    let body = quadruped_with(1, 0.4);
    let robot = Vec3::new(25.0, 0.0, 0.0);
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::X, 23);
    let mood = Mood {
        alarm: 1.0,
        threat: robot,
        stance: Stance::Attack,
        ..Mood::default()
    };
    let aim = Aim {
        place: Vec3::ZERO,
        flow: Vec3::ZERO,
        excite: 1.0,
    };
    let pace = pace(robot);
    let flat = |_: Vec2| 0.0;
    let mut closest = f32::MAX;
    for _ in 0..360 {
        let d = drive(&motion, &body, (&aim, &mood), pace, &flat);
        motion.step(&body, &d, &flat, DT);
        closest = closest.min((robot - motion.root.at).xz().length());
    }
    let gap = (robot - motion.root.at).xz().length();
    assert!(gap < STRIKE + 0.5, "stopped {gap} m from the robot");
    assert!(closest > 0.3, "ran through the robot, {closest} m from it");
}
