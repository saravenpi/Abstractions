//! Tests of hits: the sweep meets a part's box in its own frame, the sweet spot, the blow and the
//! kickback, and the hitstop frames.

use bevy::prelude::*;

use super::hit::{Bounds, enter, radius, sweet, touch};
use super::sense::landed;
use super::stage::{blade, club};
use super::stop::frames;
use super::tune::{KICK, SWEET_FLOOR};

/// A unit box at the origin.
const BOX: Bounds = Bounds {
    centre: Vec3::ZERO,
    half: Vec3::splat(0.5),
};

/// A segment through the box enters at its face, one beside it misses, and padding catches it.
#[test]
fn the_slab_test_finds_the_face() {
    let at = enter(
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        &BOX,
        Vec3::ZERO,
    );
    assert!((at.expect("goes through") - 0.375).abs() < 1e-5);
    let beside = (Vec3::new(-2.0, 0.7, 0.0), Vec3::new(2.0, 0.7, 0.0));
    assert_eq!(enter(beside.0, beside.1, &BOX, Vec3::ZERO), None);
    assert!(enter(beside.0, beside.1, &BOX, Vec3::splat(0.25)).is_some());
    assert_eq!(
        enter(
            Vec3::new(-2.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            &BOX,
            Vec3::ZERO
        ),
        None
    );
}

/// The sweep is tested in the frame of a moved, turned and stretched part.
#[test]
fn the_sweep_meets_a_placed_part() {
    let part = GlobalTransform::from(
        Transform::from_xyz(5.0, 1.0, 0.0)
            .with_rotation(Quat::from_rotation_y(0.7))
            .with_scale(Vec3::new(0.4, 1.0, 2.0)),
    );
    let grip = Vec3::new(5.0, 1.0, 3.0);
    let was = [grip, Vec3::new(4.0, 1.0, 2.0)];
    let now = [grip, Vec3::new(6.0, 1.0, 2.0)];
    assert!(touch(was, now, &part, &BOX, 0.0).is_none());
    let deep = [grip, Vec3::new(5.0, 1.0, 0.0)];
    let at = touch(was, deep, &part, &BOX, 0.0).expect("the head reaches into it");
    assert!(at.distance(Vec3::new(5.0, 1.0, 0.0)) < 1.5);
}

/// The head is the sweet spot: all the energy there, less towards the grip, never under the floor.
#[test]
fn the_head_is_the_sweet_spot() {
    assert_eq!(sweet(0.6, 0.6), 1.0);
    assert!(sweet(0.4, 0.6) < 1.0 && sweet(0.4, 0.6) > sweet(0.2, 0.6));
    assert_eq!(sweet(0.0, 0.6), SWEET_FLOOR);
    let r = radius(
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.3, 0.4, 0.0),
    );
    assert!((r - 0.4).abs() < 1e-6);
}

/// A blow at the head carries the swing energy; one near the grip less; the kickback stun is at
/// most 0.1 s and shorter when the blow gives more of its energy away.
#[test]
fn a_blow_carries_the_swing() {
    let held = club();
    let tip = Vec3::Y * held.build.reach();
    let (head, stun) = landed(&held, [Vec3::ZERO, tip], Vec3::Y * held.build.shaft.length);
    assert!((head.energy - held.swing.energy).abs() / held.swing.energy < 1e-4);
    let (near, _) = landed(&held, [Vec3::ZERO, tip], Vec3::Y * 0.05);
    assert!(near.energy < head.energy);
    assert!((0.0..=KICK).contains(&stun));
    let (_, light) = landed(&blade(), [Vec3::ZERO, tip], tip);
    assert!((0.0..=KICK).contains(&light));
}

/// The hitstop lasts 3 + 0.4·√E frames, from 3 to 12.
#[test]
fn the_hitstop_grows_with_energy() {
    assert_eq!(frames(0.0), 3.0);
    assert!((frames(100.0) - 7.0).abs() < 1e-5);
    assert_eq!(frames(10_000.0), 12.0);
}
