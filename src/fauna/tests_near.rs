//! Tests of the live herd marks: a herd is marked only while its animals are drawn, at the middle of
//! them, the mark moves with them every frame, flyers too, and it goes the frame they are gone.

use bevy::math::DVec3;

use super::near::{Animal, marks};
use super::rig::{herd_of, planet};
use super::tune::{MARK_APART, MARK_SIZE};
use crate::life::Species;

/// Returns the unit east tangent at unit direction `dir`, away from the poles.
fn east(dir: DVec3) -> DVec3 {
    DVec3::Y.cross(dir).normalize()
}

/// Returns true when the herds of `species` are worth a mark by their size.
fn big(species: &Species) -> bool {
    species.body.length >= MARK_SIZE
}

/// Returns two drawn animals of herd `id` of species `index` standing 6 m either side of `at`, raised
/// `lift` metres as flyers are.
fn pair(id: u64, index: u16, (at, lift): (DVec3, f64)) -> [Animal; 2] {
    let (side, up) = (east(at) * 6.0, at.normalize() * lift);
    [(id, index, at + side + up), (id, index, at - side + up)]
}

/// A herd whose animals are drawn is marked at the middle of them; with none of its animals drawn it
/// has no mark, however near its herd is.
#[test]
fn a_herd_is_marked_only_while_drawn() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd_of(&globe, &life, body, big).expect("a herd worth a mark");
    let at = herd.centre * globe.radius();
    let robot = (at + east(at) * 30.0, globe.radius());
    let shown = marks(&life, &pair(herd.id, herd.species, (at, 0.0)), robot);
    assert_eq!(shown.len(), 1);
    assert!(
        shown[0].at.distance(at) < 1e-6,
        "the mark is off the middle of the animals"
    );
    assert!(
        (shown[0].metres - 30.0).abs() < 1.0,
        "{} m",
        shown[0].metres
    );
    assert!(
        marks(&life, &[], robot).is_empty(),
        "marked with no animal drawn"
    );
}

/// As the animals of a herd move, its mark moves with them, frame by frame, also when they fly 12 m
/// over the ground: it is never more than a millimetre off their middle.
#[test]
fn a_mark_moves_with_its_animals_flyers_too() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd_of(&globe, &life, body, big).expect("a herd worth a mark");
    let start = herd.centre * globe.radius();
    let robot = (start, globe.radius());
    for frame in 0..60 {
        let at = start + east(start) * (0.5 * f64::from(frame));
        let lift = 12.0 * (f64::from(frame) * 0.1).sin().abs();
        let shown = marks(&life, &pair(herd.id, herd.species, (at, lift)), robot);
        let middle = at + at.normalize() * lift;
        let gap = shown[0].at.distance(middle);
        assert!(
            gap < 1e-3,
            "frame {frame}: the mark is {gap} m from its animals"
        );
    }
}

/// The frame a herd's animals are no longer drawn its mark is gone, while the mark of a herd still
/// drawn stays; two herds closer than [`MARK_APART`] share one mark, the nearer one's.
#[test]
fn a_mark_goes_the_frame_its_herd_leaves() {
    let (body, globe, life) = planet(true);
    let life = life.expect("a living planet");
    let herd = herd_of(&globe, &life, body, big).expect("a herd worth a mark");
    let at = herd.centre * globe.radius();
    let far = at + east(at) * (MARK_APART + 40.0);
    let robot = (at, globe.radius());
    let both: Vec<Animal> = [
        pair(herd.id, herd.species, (at, 0.0)),
        pair(herd.id + 1, herd.species, (far, 0.0)),
    ]
    .concat();
    assert_eq!(marks(&life, &both, robot).len(), 2);
    let left = pair(herd.id + 1, herd.species, (far, 0.0));
    let shown = marks(&life, &left, robot);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].id, herd.id + 1, "the herd that left kept its mark");
    let close = at + east(at) * (0.5 * MARK_APART);
    let crowd: Vec<Animal> = [
        pair(herd.id, herd.species, (at, 0.0)),
        pair(herd.id + 1, herd.species, (close, 0.0)),
    ]
    .concat();
    let shown = marks(&life, &crowd, robot);
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].id, herd.id, "the nearer herd lost its mark");
}
