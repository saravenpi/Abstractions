//! Tests of the reaction distances: a robot can walk to about ten metres of a calm grazer, which
//! looks up at about twenty, small animals let it nearer, hunters are bolder, and a robot that
//! stands still is let nearer than one that walks.

use bevy::math::Vec3;

use super::react::{Act, Mood, Temper};
use crate::life::Guild;

/// Returns what an animal of `temper` does after `seconds` with a threat `metres` away that walks
/// past at `speed` metres per second.
fn after(temper: &Temper, metres: f32, speed: f32, seconds: f32) -> Act {
    let mut mood = Mood::default();
    let dt = 0.05;
    for step in 0..(seconds / dt) as u32 {
        let at = Vec3::new(metres, 0.0, speed * dt * step as f32);
        mood.feel(temper, Some((at, metres)), dt);
    }
    mood.act()
}

/// A 1 m grazer lets a walking robot come to 12 m without running, runs at 9 m, and looks up at
/// 18 m.
#[test]
fn a_grazer_lets_a_robot_near() {
    let grazer = Temper::new(Guild::Grazer, 1.0);
    println!(
        "grazer: notice {:.1} m, flee {:.1} m",
        grazer.notice, grazer.flee
    );
    assert!((8.0..=12.0).contains(&grazer.flee), "{grazer:?}");
    assert!((18.0..=24.0).contains(&grazer.notice), "{grazer:?}");
    assert_eq!(after(&grazer, 12.5, 1.5, 3.0), Act::Alert);
    assert_eq!(after(&grazer, 9.0, 1.5, 3.0), Act::Flee);
    assert_eq!(after(&grazer, 30.0, 1.5, 3.0), Act::Graze);
}

/// A robot that stands still 8 m from a grazer only makes it look up.
#[test]
fn a_still_robot_does_not_scare() {
    let grazer = Temper::new(Guild::Grazer, 1.0);
    assert_eq!(after(&grazer, 8.0, 0.0, 5.0), Act::Alert);
    assert_eq!(after(&grazer, 8.0, 1.5, 5.0), Act::Flee);
}

/// A small animal flees nearer than a big one, and a hunter lets the robot nearer than a grazer of
/// its size.
#[test]
fn small_and_bold_animals_flee_nearer() {
    let small = Temper::new(Guild::Grazer, 0.25);
    let big = Temper::new(Guild::Grazer, 2.5);
    let hunter = Temper::new(Guild::Hunter, 1.0);
    let grazer = Temper::new(Guild::Grazer, 1.0);
    assert!(small.flee < grazer.flee && grazer.flee < big.flee);
    assert!(small.flee >= 5.0, "{small:?}");
    assert!(hunter.flee < 0.5 * grazer.flee, "{hunter:?}");
    assert!(hunter.notice < grazer.notice, "{hunter:?}");
}
