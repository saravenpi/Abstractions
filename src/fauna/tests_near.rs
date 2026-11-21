//! Tests of the herd marks: a herd is marked only once the robot has seen its drawn animals, in view,
//! within sight and with no ground between, and the mark follows the herd until it fades out.

use bevy::math::DVec3;

use super::near::{Eye, Nearby, remember, sees};
use super::rig::{herd, planet};
use super::tune::{MEMORY, SIGHT};
use crate::life::Life;
use crate::life::herd::Herd;
use crate::universe::BodyId;
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

/// A herd whose drawn animals are in view and within sight is marked at the middle of them; one behind
/// the robot, one too far, one hidden under the ground and one with no animal drawn are not.
#[test]
fn only_seen_herds_are_marked() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd(&globe, &life, body).expect("a herd");
    let drawn = [(herd.id, ground(&globe, herd.centre, 0.0))];
    let mark = |eye: Eye, drawn: &[(u64, DVec3)]| {
        let mut nearby = Nearby::default();
        remember(
            &mut nearby,
            (&life, &globe, Some(body)),
            (&[herd], drawn),
            (eye, 0.0),
        );
        nearby.herds
    };
    let seen = mark(eye(&globe, &herd, 30.0, true), &drawn);
    assert_eq!(seen.len(), 1);
    assert!(seen[0].at.distance(drawn[0].1) < 1e-6);
    assert!((seen[0].metres - 30.0).abs() < 2.0, "{} m", seen[0].metres);
    assert!(mark(eye(&globe, &herd, 30.0, false), &drawn).is_empty());
    assert!(mark(eye(&globe, &herd, SIGHT + 20.0, true), &drawn).is_empty());
    assert!(
        mark(eye(&globe, &herd, 30.0, true), &[]).is_empty(),
        "marked with no animal drawn"
    );
    let near = eye(&globe, &herd, 60.0, true);
    let buried = ground(&globe, herd.centre, -40.0);
    assert!(!sees(&globe, near, buried));
}

/// Returns the marks after the robot saw `herd` of `life` on `globe`, the planet `body`, from 30 m with
/// one animal drawn at its centre, at universe time 0.
fn seen_once(globe: &Globe, (life, body): (&Life, BodyId), herd: &Herd) -> Nearby {
    let mut nearby = Nearby::default();
    let here = [(herd.id, ground(globe, herd.centre, 0.0))];
    let eye = eye(globe, herd, 30.0, true);
    remember(
        &mut nearby,
        (life, globe, Some(body)),
        (&[*herd], &here),
        (eye, 0.0),
    );
    nearby
}

/// Returns `herd` moved 200 m east on `globe`.
fn moved(globe: &Globe, herd: &Herd) -> Herd {
    Herd {
        centre: (herd.centre + east(herd.centre) * 200.0 / globe.radius()).normalize(),
        ..*herd
    }
}

/// Once out of view and no longer drawn, a mark follows its herd to the centre its animals are placed
/// around when drawn again, within a metre, and fades out [`MEMORY`] seconds after it was last seen.
#[test]
fn a_mark_follows_its_herd_when_not_drawn() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd(&globe, &life, body).expect("a herd");
    let mut nearby = seen_once(&globe, (&life, body), &herd);
    let (moved, away) = (moved(&globe, &herd), eye(&globe, &herd, 30.0, false));
    let world = (&life, &*globe, Some(body));
    remember(&mut nearby, world, (&[moved], &[]), (away, 60.0));
    assert_eq!(nearby.herds.len(), 1);
    let gap = nearby.herds[0]
        .at
        .distance(ground(&globe, moved.centre, 0.0));
    assert!(gap < 1.0, "the mark is {gap} m from the herd");
    assert!((nearby.herds[0].fresh(60.0) - (1.0 - 60.0 / MEMORY) as f32).abs() < 1e-4);
    remember(&mut nearby, world, (&[moved], &[]), (away, MEMORY + 1.0));
    assert!(nearby.herds.is_empty());
}

/// While its animals are drawn, a mark stands at the middle of them, within a metre, and not among the
/// animals of another herd; once its herd is no longer read, as when every member died, the mark goes.
#[test]
fn a_mark_stands_among_its_drawn_animals() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd(&globe, &life, body).expect("a herd");
    let mut nearby = seen_once(&globe, (&life, body), &herd);
    let (moved, away) = (moved(&globe, &herd), eye(&globe, &herd, 30.0, false));
    let world = (&life, &*globe, Some(body));
    let (at, side) = (ground(&globe, moved.centre, 0.0), east(moved.centre) * 6.0);
    let animals = [
        (herd.id, at + side),
        (herd.id, at - side),
        (herd.id + 1, at + side * 40.0),
    ];
    remember(&mut nearby, world, (&[moved], &animals), (away, 10.0));
    let gap = nearby.herds[0].at.distance(at);
    assert!(
        gap < 1.0,
        "the mark is {gap} m from the middle of its animals"
    );
    remember(&mut nearby, world, (&[], &[]), (away, 20.0));
    assert!(
        nearby.herds.is_empty(),
        "a herd no longer read kept its mark"
    );
}
