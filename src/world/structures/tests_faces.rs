//! Tests that no two faces of a structure fight for the same plane.

use super::camp::{Layout, draw_layout};
use super::measure::fights;
use super::probe::Shot;
use super::site::Kind;
use super::tests::built;
use super::tests_doors::{SITE, every_house};
use super::{DRAW, Terrain};

/// Describes the first fighting pairs of `shot` by the boxes around their triangles.
fn report(shot: &Shot, pairs: &[(usize, usize)]) -> String {
    let around = |t: usize| shot.bounds(t * 3, t * 3 + 3);
    let lines = pairs
        .iter()
        .take(4)
        .map(|(a, b)| format!("{:?} with {:?}", around(*a), around(*b)));
    lines.collect::<Vec<_>>().join("; ")
}

/// Asserts that `shot` holds no pair of overlapping faces in one plane facing the same way.
fn check(shot: &Shot, what: &str) {
    let pairs = fights(&shot.verts);
    assert!(
        pairs.is_empty(),
        "{what}: {} fighting pairs: {}",
        pairs.len(),
        report(shot, &pairs)
    );
}

/// No house of any kind, style or lot variation has two overlapping faces in one plane facing the same way.
#[test]
fn house_faces_do_not_fight() {
    every_house(6, |shot, _, what| check(shot, what));
}

/// No small structure and no camp layout has two overlapping faces in one plane facing the same way.
#[test]
fn structure_faces_do_not_fight() {
    let t = Terrain::new(1);
    for kind in Kind::SMALL.into_iter().filter(|k| *k != Kind::Camp) {
        let mut b = built(&t, kind, 3);
        DRAW[kind as usize](&mut b);
        check(&Shot::of(&b, SITE.0, SITE.1), &format!("{kind:?}"));
    }
    for layout in Layout::ALL {
        let mut b = built(&t, Kind::Camp, 3);
        draw_layout(&mut b, layout);
        check(&Shot::of(&b, SITE.0, SITE.1), &format!("{layout:?}"));
    }
}
