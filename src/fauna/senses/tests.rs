//! Tests of the senses: the meter fills in the times of its bands, slower for a still robot and
//! faster for a fast one, drains slowly, is blocked by a ridge, and catches scent only downwind; a
//! lookout notices from farther.

use bevy::math::{DVec3, Vec2, Vec3};

use super::feel::{Air, Senser, Threat, perceive};
use super::meter::Meter;
use super::tune::{LOOKOUT, NOSE_GRAZER};
use crate::fauna::place::Spot;

/// Returns a grazer at the origin of a frame on a planet of 1000 km, with senses of range `keen`.
pub(super) fn grazer(keen: f32) -> Senser {
    Senser {
        spot: Spot::at(DVec3::Y, 1.0e6),
        at: Vec3::ZERO,
        eye: 1.0,
        keen,
        nose: NOSE_GRAZER,
        herd: 1,
        fleeing: false,
    }
}

/// Returns a robot at `local` in the frame of `me`, going `speed` metres per second at loudness
/// `loud`.
pub(super) fn robot(me: &Senser, local: Vec3, (speed, loud): (f32, f32)) -> Threat {
    Threat {
        pos: me.spot.planet(local),
        speed,
        loud,
        mass: 80.0,
        key: 9,
        herd: 0,
        robot: true,
        crouch: 0.0,
    }
}

/// Returns the seconds `me` takes to be fully aware of `threat` over `ground` with `wind`, or
/// nothing within 20 s.
pub(super) fn fill(
    me: &Senser,
    threat: Threat,
    ground: &dyn Fn(Vec2) -> f32,
    wind: DVec3,
) -> Option<f32> {
    let mut meter = Meter::default();
    let dt = 0.01;
    let air = Air { wind, ground, dt };
    (1..=2000).find_map(|step| {
        let aware = perceive(&mut meter, me, &[threat], &air).map_or(0.0, |b| b.1);
        (aware >= 1.0).then_some(step as f32 * dt)
    })
}

/// The meter fills in 0.3 s under 10 m, 1.5 s at 10-30 m and 4 s at 30-60 m for a walking robot,
/// and never past 60 m when the robot is quiet and there is no wind; a still robot fills it at half
/// the rate, a fast one at twice.
#[test]
fn the_meter_fills_by_band() {
    let me = grazer(1.0);
    let flat = |_: Vec2| 0.0;
    let near = |s: Option<f32>, goal: f32| s.is_some_and(|s| (s - goal).abs() <= 0.15 * goal);
    let at = |m: f32| robot(&me, Vec3::new(m, 0.0, 0.0), (1.5, 0.0));
    assert!(near(fill(&me, at(5.0), &flat, DVec3::ZERO), 0.3));
    assert!(near(fill(&me, at(20.0), &flat, DVec3::ZERO), 1.5));
    assert!(near(fill(&me, at(45.0), &flat, DVec3::ZERO), 4.0));
    assert_eq!(fill(&me, at(70.0), &flat, DVec3::ZERO), None);
    let moving = |speed: f32| robot(&me, Vec3::new(20.0, 0.0, 0.0), (speed, 0.0));
    assert!(near(fill(&me, moving(0.0), &flat, DVec3::ZERO), 3.0));
    assert!(near(fill(&me, moving(6.0), &flat, DVec3::ZERO), 0.75));
}

/// The meter drains about 0.1 per second once the robot is gone.
#[test]
fn the_meter_drains_slowly() {
    let me = grazer(1.0);
    let flat = |_: Vec2| 0.0;
    let mut meter = Meter::default();
    meter.raise(9, 1.0);
    let air = Air {
        wind: DVec3::ZERO,
        ground: &flat,
        dt: 0.1,
    };
    for _ in 0..50 {
        perceive(&mut meter, &me, &[], &air);
    }
    assert!((meter.of(9) - 0.5).abs() < 0.02, "{}", meter.of(9));
}

/// A ridge between the grazer and a quiet robot hides it; a walking one is still heard near.
#[test]
fn a_ridge_blocks_sight() {
    let me = grazer(1.0);
    let ridge = |p: Vec2| {
        if (10.0..14.0).contains(&p.x) {
            6.0
        } else {
            0.0
        }
    };
    let quiet = robot(&me, Vec3::new(40.0, 0.0, 0.0), (1.5, 0.0));
    assert_eq!(fill(&me, quiet, &ridge, DVec3::ZERO), None);
    let heard = robot(&me, Vec3::new(15.0, 0.0, 0.0), (1.5, 1.0));
    assert!(fill(&me, heard, &ridge, DVec3::ZERO).is_some());
}

/// Scent carries a robot 70 m upwind to the grazer, not one 70 m downwind of it.
#[test]
fn scent_carries_downwind() {
    let me = grazer(1.0);
    let flat = |_: Vec2| 0.0;
    let wind = me.spot.rot * DVec3::new(-3.0, 0.0, 0.0);
    let upwind = robot(&me, Vec3::new(70.0, 0.0, 0.0), (1.5, 0.0));
    let downwind = robot(&me, Vec3::new(-70.0, 0.0, 0.0), (1.5, 0.0));
    assert!(fill(&me, upwind, &flat, wind).is_some());
    assert_eq!(fill(&me, downwind, &flat, wind), None);
}

/// A lookout sees a quiet robot at 75 m, past the 60 m of the others.
#[test]
fn the_lookout_sees_farther() {
    let flat = |_: Vec2| 0.0;
    let watch = grazer(LOOKOUT);
    let far = robot(&watch, Vec3::new(75.0, 0.0, 0.0), (1.5, 0.0));
    assert!(fill(&watch, far, &flat, DVec3::ZERO).is_some());
    assert_eq!(fill(&grazer(1.0), far, &flat, DVec3::ZERO), None);
}
