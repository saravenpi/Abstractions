//! Chains keep their lengths and bend limits, and tails lag behind the body.

use std::f32::consts::PI;

use bevy::math::{Quat, Vec3};

use super::chain::{follow, straight};
use super::fixtures::quadruped_with;
use super::frame::{Role, chains};
use super::rig::Motion;
use super::tests_plans::drive;
use crate::life::body::Body;

/// Returns the largest bend between successive segments of `points`, the first from `base`.
fn worst_bend(points: &[Vec3], base: Vec3) -> f32 {
    let mut prev = base.normalize();
    let mut worst: f32 = 0.0;
    for pair in points.windows(2) {
        let dir = (pair[1] - pair[0]).normalize();
        worst = worst.max(prev.angle_between(dir));
        prev = dir;
    }
    worst
}

/// A chain whose head turns 180 degrees keeps every length to 1e-4 and every joint in its limit.
#[test]
fn chain_survives_a_half_turn_of_its_head() {
    let lengths = [0.3; 6];
    let limit = 0.5;
    let mut points = straight(Vec3::ZERO, Vec3::Z, &lengths);
    for step in 0..=90 {
        let turn = Quat::from_rotation_y(PI * step as f32 / 90.0);
        let anchor = turn * Vec3::new(0.0, 0.0, -0.4);
        follow(&mut points, &lengths, (anchor, turn * Vec3::Z), limit);
        for (pair, length) in points.windows(2).zip(lengths) {
            assert!(
                (pair[0].distance(pair[1]) - length).abs() < 1e-4,
                "step {step}"
            );
        }
        assert!(
            worst_bend(&points, turn * Vec3::Z) <= limit + 1e-4,
            "step {step}"
        );
    }
}

/// Asserts that every strand of `motion` keeps the segment lengths of its chain on `body`.
fn assert_lengths(motion: &Motion, body: &Body) {
    for (strand, (_, chain)) in motion.strands.iter().zip(chains(body)) {
        for (pair, length) in strand.points.windows(2).zip(&chain.lengths) {
            assert!((pair[0].distance(pair[1]) - length).abs() < 1e-4);
        }
    }
}

/// A quadruped walking a curve keeps its neck and tail lengths, and its tail trails behind it.
#[test]
fn tail_and_neck_follow_the_body() {
    let body = quadruped_with(4, 0.35);
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 2);
    for frame in 0..300 {
        let way = Quat::from_rotation_y(frame as f32 * 0.02) * Vec3::NEG_Z;
        motion.step(&body, &drive(way * 1.5, 0.0), &|_| 0.0, 1.0 / 30.0);
        assert_lengths(&motion, &body);
    }
    let tail = &motion.strands[1].points;
    let back = motion.root.turn * Vec3::Z;
    assert!(
        (tail[tail.len() - 1] - tail[0]).dot(back) > 0.3,
        "tail not behind"
    );
}

/// A standing quadruped turns its head toward a point on its left as far as the clamp allows.
#[test]
fn head_turns_toward_a_point() {
    let body = quadruped_with(2, 0.3);
    let mut motion = Motion::new(&body, Vec3::ZERO, Vec3::NEG_Z, 2);
    let mut asked = drive(Vec3::ZERO, 0.0);
    asked.look = Some(Vec3::new(-4.0, 2.0, -1.0));
    for _ in 0..60 {
        motion.step(&body, &asked, &|_| 0.0, 1.0 / 30.0);
    }
    let yaw = motion.look.at.x;
    assert!(yaw > 0.9 && yaw <= 1.2 + 1e-3, "head turned {yaw}");
    let head = motion
        .frame
        .roles
        .iter()
        .position(|r| *r == Role::Head)
        .unwrap_or(0);
    let facing = motion.pose(&body)[head].1 * Vec3::NEG_Z;
    assert!(facing.x < -0.7, "head faces {facing}");
}
