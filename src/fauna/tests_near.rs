//! Tests of the herd marks: a herd is marked only once the robot has seen it, in view, within sight
//! and with no ground between, and the mark stays where it was last seen until it fades out.

use bevy::math::DVec3;

use super::near::{Eye, Nearby, remember, sees};
use super::rig::{herd, planet};
use super::tune::{MEMORY, SIGHT};
use crate::life::herd::Herd;
use crate::world::Globe;

/// Returns the unit east tangent at unit direction `dir`, away from the poles.
fn east(dir: DVec3) -> DVec3 {
    DVec3::Y.cross(dir).normalize()
}

/// Returns the planet position of the ground under unit direction `dir` of `globe`, raised `lift`.
fn ground(globe: &Globe, dir: DVec3, lift: f64) -> DVec3 {
    dir * (globe.radius() + f64::from(globe.height(dir).max(0.0)) + lift)
}

/// Returns the eye of a robot standing `metres` east of the centre of `herd` on `globe`, looking at
/// it when `facing`, away from it otherwise.
fn eye(globe: &Globe, herd: &Herd, metres: f64, facing: bool) -> Eye {
    let dir = (herd.centre + east(herd.centre) * metres / globe.radius()).normalize();
    let at = ground(globe, dir, 1.6);
    let towards = (ground(globe, herd.centre, 1.0) - at).normalize();
    Eye {
        at,
        ahead: if facing { towards } else { -towards },
    }
}

/// A herd in view and within sight is marked where it stands; one behind the robot, one too far and
/// one hidden under the ground are not.
#[test]
fn only_seen_herds_are_marked() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd(&globe, &life, body).expect("a herd");
    let mark = |eye: Eye| {
        let mut nearby = Nearby::default();
        remember(
            &mut nearby,
            (&life, &globe, Some(body)),
            &[herd],
            (eye, 0.0),
        );
        nearby.herds
    };
    let seen = mark(eye(&globe, &herd, 30.0, true));
    assert_eq!(seen.len(), 1);
    assert!(seen[0].at.normalize().distance(herd.centre) < 1e-9);
    assert!((seen[0].metres - 30.0).abs() < 2.0, "{} m", seen[0].metres);
    assert!(mark(eye(&globe, &herd, 30.0, false)).is_empty());
    assert!(mark(eye(&globe, &herd, SIGHT + 20.0, true)).is_empty());
    let near = eye(&globe, &herd, 60.0, true);
    let buried = ground(&globe, herd.centre, -40.0);
    assert!(!sees(&globe, near, buried));
}

/// Once out of view, a herd stays marked where it was last seen though it walks on, and its mark
/// fades out after [`MEMORY`] seconds.
#[test]
fn a_mark_stays_where_the_herd_was_seen() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd(&globe, &life, body).expect("a herd");
    let world = (&life, &*globe, Some(body));
    let mut nearby = Nearby::default();
    remember(
        &mut nearby,
        world,
        &[herd],
        (eye(&globe, &herd, 30.0, true), 0.0),
    );
    let first = nearby.herds[0].at;
    let moved = Herd {
        centre: (herd.centre + east(herd.centre) * 200.0 / globe.radius()).normalize(),
        ..herd
    };
    let away = eye(&globe, &herd, 30.0, false);
    remember(&mut nearby, world, &[moved], (away, 60.0));
    assert_eq!(nearby.herds.len(), 1);
    assert_eq!(nearby.herds[0].at, first);
    assert!((nearby.herds[0].fresh(60.0) - (1.0 - 60.0 / MEMORY) as f32).abs() < 1e-4);
    remember(&mut nearby, world, &[moved], (away, MEMORY + 1.0));
    assert!(nearby.herds.is_empty());
}
