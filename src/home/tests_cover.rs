//! Tests that every part of the base is solid where it is drawn: no lit triangle large enough to run into
//! lies farther from a solid box, or higher above the ground, than a body is wide.

use bevy::math::Affine3A;
use bevy::prelude::*;

use super::block::{BULK, Block, pruned};
use super::cover::{NEAR, SIZE, THIN, bare, triangles};
use super::plan::plan;
use super::soup::Soup;
use super::spawn::solid;
use super::tests::base;

/// On twelve worlds, every lit triangle of the base that is neither a rod nor a cable lies within reach of
/// a solid box, or on the ground: the container with its vent, dish and legs, the station with its canopy,
/// posts, bench, crate and mast, the corner posts of the pad and the slabs of the path.
#[test]
fn every_part_of_the_base_is_solid_where_drawn() {
    for seed in 1..=12 {
        let (base, globe) = base(seed);
        let lot = base.lot(&globe);
        let plan = plan(&lot);
        let tris = triangles(&plan.lit);
        let grounded = |p: Vec3| p.y <= lot.crest(p.x, p.z) + NEAR;
        let found = bare(&tris, &solid(&plan.blocks), |_, p| grounded(p));
        let (lo, hi) = found
            .iter()
            .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), (_, p)| {
                (lo.min(*p), hi.max(*p))
            });
        assert!(
            found.is_empty(),
            "seed {seed}: {} drawn points have no solid box, in {lo:.2} .. {hi:.2}",
            found.len()
        );
    }
}

/// A box of the base is solid from a size no larger than the size from which the cover test counts a drawn
/// triangle: nothing falls between the two rules.
#[test]
fn solid_sizes_match_the_cover_test() {
    assert!(BULK.0 <= SIZE && BULK.1 <= THIN);
    assert!(Block::between(Vec3::ZERO, Vec3::new(0.12, 2.5, 0.12)).bulky());
    assert!(Block::between(Vec3::ZERO, Vec3::new(4.4, 0.1, 3.6)).bulky());
    assert!(!Block::between(Vec3::ZERO, Vec3::new(0.06, 3.0, 0.06)).bulky());
    assert!(!Block::between(Vec3::ZERO, Vec3::new(0.2, 0.2, 0.2)).bulky());
}

/// The box of a drawn bar is the bar: every vertex of the bar lies on a corner of its box, moved or not.
#[test]
fn bars_get_the_box_they_are_drawn_as() {
    let pose =
        Affine3A::from_rotation_translation(Quat::from_rotation_y(0.7), Vec3::new(3.0, 1.0, -2.0));
    let ends = [
        (Vec3::ZERO, Vec3::Y * 2.5),
        (Vec3::new(1.0, 0.2, 0.5), Vec3::new(-0.5, 1.4, 2.0)),
    ];
    for (a, b) in ends {
        let mut soup = Soup::default();
        soup.beam(a, b, 0.09, [1.0; 3]);
        let block = soup.boxes()[0].moved(pose);
        let corners = block.corners();
        let mut moved = Soup::default();
        moved.merge(&soup, pose);
        for tri in triangles(&moved.into_mesh()) {
            let near = |p: Vec3| {
                corners
                    .iter()
                    .map(|c| c.distance(p))
                    .fold(f32::MAX, f32::min)
            };
            assert!(tri.into_iter().all(|p| near(p) < 1e-4), "{a} to {b}");
        }
    }
}

/// A plate drawn on a box is left to the box, and two boxes apart are both kept, whatever their order.
#[test]
fn plates_are_left_to_the_box_they_lie_on() {
    let hull = Block::between(Vec3::ZERO, Vec3::new(6.0, 2.8, 2.6));
    let hatch = Block::between(Vec3::new(1.0, 0.3, 2.6), Vec3::new(2.1, 2.3, 2.68));
    let vent = Block::between(Vec3::new(1.4, 2.8, 0.8), Vec3::new(2.3, 2.92, 1.6));
    let crate_ = Block::between(Vec3::new(9.0, 0.0, 0.0), Vec3::new(9.8, 0.8, 0.8));
    for list in [
        vec![hull, hatch, vent, crate_],
        vec![crate_, vent, hatch, hull],
    ] {
        let kept = pruned(list);
        assert_eq!(kept.len(), 3);
        assert!(!kept.contains(&hatch) && kept.contains(&vent));
    }
    assert_eq!(pruned(vec![hull, hull]).len(), 1);
}
