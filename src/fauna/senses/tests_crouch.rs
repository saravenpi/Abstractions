//! Tests of the crouch: a crouched robot fills the meter at half the rate, never below a quarter,
//! walks quieter and hides behind ground a standing one is seen over.

use bevy::math::{DVec3, Vec2, Vec3};

use super::cues::loudness;
use super::feel::Threat;
use super::meter::stealth;
use super::tests::{fill, grazer, robot};

/// Returns `threat` crouched all the way down.
fn crouched(threat: Threat) -> Threat {
    Threat {
        crouch: 1.0,
        ..threat
    }
}

/// A robot walking 20 m away fills the meter in 1.5 s standing and in 3 s crouched.
#[test]
fn a_crouched_robot_fills_the_meter_at_half_rate() {
    let me = grazer(1.0);
    let flat = |_: Vec2| 0.0;
    let walking = robot(&me, Vec3::new(20.0, 0.0, 0.0), (1.5, 0.0));
    let up = fill(&me, walking, &flat, DVec3::ZERO).unwrap();
    let down = fill(&me, crouched(walking), &flat, DVec3::ZERO).unwrap();
    assert!((down / up - 2.0).abs() < 0.1, "{up} s against {down} s");
}

/// Crouching and creeping together keep a quarter of the rate, and no less.
#[test]
fn the_fill_rate_never_falls_under_a_quarter() {
    assert_eq!(stealth(1.5, 0.0, 0.0), 1.0);
    assert_eq!(stealth(1.5, 0.0, 1.0), 0.5);
    assert_eq!(stealth(0.0, 0.0, 0.0), 0.5);
    assert_eq!(stealth(0.0, 0.0, 1.0), 0.25);
    assert_eq!(stealth(6.0, 0.0, 1.0), 1.0);
}

/// A crouched robot makes less noise at the same speed.
#[test]
fn a_crouched_robot_is_quieter() {
    assert!(loudness(1.5, 0.0, 0.0, 1.0) < loudness(1.5, 0.0, 0.0, 0.0));
}

/// A low rise the standing robot is seen over hides the crouched one.
#[test]
fn low_ground_hides_a_crouched_robot() {
    let me = grazer(1.0);
    let rise = |p: Vec2| if (7.0..17.0).contains(&p.x) { 0.9 } else { 0.0 };
    let standing = robot(&me, Vec3::new(40.0, 0.0, 0.0), (1.5, 0.0));
    assert!(fill(&me, standing, &rise, DVec3::ZERO).is_some());
    assert_eq!(fill(&me, crouched(standing), &rise, DVec3::ZERO), None);
}
