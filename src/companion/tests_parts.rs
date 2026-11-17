//! Tests of the lean, the hands, the light and the celebration of new planets.

use bevy::prelude::*;

use super::brain::Brain;
use super::limbs::{hand_pose, trail};
use super::motion::lean;
use super::pose::Pose;
use super::sense::CompanionSense;
use super::shine::{lumens, radiance};
use super::tune::{HAND_LAG, LIGHT_DAY_SHARE, LIGHT_POWER, TILT};
use crate::universe::BodyId;

/// The body leans into its travel, the hands trail behind it, both within their limits, and a
/// stretch moves the hands apart.
#[test]
fn lean_and_hands_stay_within_limits() {
    for speed in [0.0, 1.0, 5.0, 60.0] {
        let velocity = Vec3::new(0.3, 0.2, -1.0).normalize() * speed;
        assert!(lean(velocity).angle_between(Quat::IDENTITY) <= TILT.1 * 1.02);
        assert!(trail(velocity).length() <= HAND_LAG.1 + 1e-5);
        assert!(trail(velocity).dot(velocity) <= 0.0);
    }
    let top = lean(Vec3::NEG_Z * 3.0) * Vec3::Y;
    assert!(top.z < -0.1, "moving forward leans the top forward");
    let rest = hand_pose(1.0, Vec3::ZERO, &Pose::default(), 0.0);
    let wide = Pose {
        spread: 1.0,
        ..default()
    };
    let stretched = hand_pose(1.0, Vec3::ZERO, &wide, 0.0);
    assert!(stretched.translation.x > rest.translation.x + 0.05);
    let left = hand_pose(-1.0, Vec3::ZERO, &Pose::default(), 0.0);
    assert!((left.translation.x + rest.translation.x).abs() < 1e-5);
}

/// The light is weak, weaker by day, and its ray traced twin carries the same colour.
#[test]
fn light_is_weak_and_follows_the_night() {
    assert!((lumens(1.0) - LIGHT_POWER).abs() < 1e-3);
    assert!((lumens(0.0) - LIGHT_POWER * LIGHT_DAY_SHARE).abs() < 1e-3);
    assert!(lumens(0.5) > lumens(0.0) && lumens(0.5) < lumens(1.0));
    assert!(
        lumens(1.0) < 26_000.0 / 4.0,
        "weaker than a quarter of a torch"
    );
    let glow = radiance(Vec3::new(1.0, 0.5, 0.25), 1000.0);
    assert!(glow.red > 0.0 && (glow.green / glow.red - 0.5).abs() < 1e-4);
}

/// The planet it first appears on is no discovery; the first steps on another one are, once.
#[test]
fn celebrates_a_new_planet_once() {
    let mut brain = Brain::new(1);
    let home = BodyId::default();
    let other = BodyId {
        index: home.index + 1,
        ..home
    };
    let (on_foot, seated) = (
        CompanionSense::default(),
        CompanionSense {
            seated: true,
            ..default()
        },
    );
    assert!(!brain.discovered(&on_foot, home));
    assert!(
        !brain.discovered(&seated, other),
        "flying over is not landing"
    );
    assert!(brain.discovered(&on_foot, other));
    assert!(!brain.discovered(&on_foot, other));
    assert!(!brain.discovered(&on_foot, home));
    let told = CompanionSense {
        delight: 1,
        ..default()
    };
    assert!(brain.discovered(&told, home));
    assert!(!brain.discovered(&told, home));
}
