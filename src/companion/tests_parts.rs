//! Tests of the lean, the hands, the light and the celebration of new planets.

use bevy::prelude::*;

use super::brain::Brain;
use super::face::Expression;
use super::limbs::{hand_pose, trail};
use super::motion::lean;
use super::pose::Pose;
use super::sense::CompanionSense;
use super::shine::{self, lumens};
use super::tune::{HAND_LAG, LIGHT_POWER, TILT};
use crate::computer::Atlas;
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

/// The light is weak: in the dark it gives its set power, far less than a torch.
#[test]
fn light_is_weak() {
    assert!((lumens(0.0, 1.0) - LIGHT_POWER).abs() < 1e-3);
    assert!(
        lumens(0.0, 1.0) < 26_000.0 / 4.0,
        "weaker than a quarter of a torch"
    );
}

/// Night alone never dims the companion: awake, the sleepy face of the night glows like a calm
/// one and the light is at full power. Only a real doze dims it.
#[test]
fn night_light_dims_only_while_dozing() {
    let calm = Expression::Neutral.glow();
    assert_eq!(shine::glow(Expression::Sleepy, false), calm);
    assert!(shine::glow(Expression::Sleepy, true) < 0.7 * calm);
    assert_eq!(lumens(310.0, 1.0), LIGHT_POWER);
    for expr in Expression::ALL {
        assert!(shine::glow(expr, false) >= shine::glow(expr, true));
        assert!(
            shine::glow(expr, false) >= 0.7,
            "{expr:?} is dim while awake"
        );
    }
}

/// Returns whether the first steps on `body` are a discovery, the way the thinking system asks.
fn steps(brain: &mut Brain, sense: &CompanionSense, atlas: &mut Atlas, body: BodyId) -> bool {
    let fresh = !sense.seated && !atlas.walked.contains(&body) && atlas.walk(body);
    brain.discovered(sense, fresh)
}

/// The planet it first appears on is no discovery; the first steps on another one are, once.
#[test]
fn celebrates_a_new_planet_once() {
    let (mut brain, mut atlas) = (Brain::new(1), Atlas::default());
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
    assert!(!steps(&mut brain, &on_foot, &mut atlas, home));
    assert!(
        !steps(&mut brain, &seated, &mut atlas, other),
        "flying over is not landing"
    );
    assert!(steps(&mut brain, &on_foot, &mut atlas, other));
    assert!(!steps(&mut brain, &on_foot, &mut atlas, other));
    assert!(!steps(&mut brain, &on_foot, &mut atlas, home));
    let told = CompanionSense {
        delight: 1,
        ..default()
    };
    assert!(steps(&mut brain, &told, &mut atlas, home));
    assert!(!steps(&mut brain, &told, &mut atlas, home));
}

/// A new companion that reads the saved atlas, as after a reload, does not celebrate a planet
/// already walked on, and still celebrates the next new one.
#[test]
fn a_reloaded_companion_does_not_celebrate_again() {
    let (mut brain, mut atlas) = (Brain::new(1), Atlas::default());
    let home = BodyId::default();
    let planet = |index: u8| BodyId { index, ..home };
    let on_foot = CompanionSense::default();
    assert!(!steps(&mut brain, &on_foot, &mut atlas, home));
    assert!(steps(
        &mut brain,
        &on_foot,
        &mut atlas,
        planet(home.index + 1)
    ));
    let saved = toml::to_string(&atlas).unwrap();
    let mut loaded: Atlas = toml::from_str(&saved).unwrap();
    let mut reborn = Brain::new(2);
    assert!(!steps(
        &mut reborn,
        &on_foot,
        &mut loaded,
        planet(home.index + 1)
    ));
    assert!(!steps(&mut reborn, &on_foot, &mut loaded, home));
    assert!(steps(
        &mut reborn,
        &on_foot,
        &mut loaded,
        planet(home.index + 2)
    ));
}
