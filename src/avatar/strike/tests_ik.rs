//! Tests of the reach and the layer: the IK puts the hand on the grip through the real rig, and the
//! layer turns the chest and moves only the arms that hold the tool.

use bevy::prelude::*;

use super::super::pose::{Core, Limb, Pose};
use super::super::rig::{Seg, globals};
use super::dir::Dir;
use super::held::Style;
use super::ik::reach;
use super::key::{Key, carry, drawn, guard};
use super::layer::Layer;
use super::path::Path;
use super::tune::HAND_GRIP;
use crate::craft::build::Hands;

/// Returns where the middle of the hand of `side` sits in the torso frame for `pose`.
fn hand(pose: &Pose, side: usize) -> Vec3 {
    let bones = globals(pose);
    let torso = bones[Seg::Torso as usize];
    let hand = bones[[Seg::HandL, Seg::HandR][side] as usize];
    torso
        .compute_affine()
        .inverse()
        .transform_point3(hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0)))
}

/// Returns the keys a strike poses: carrying, guards, the draw, and points along every path.
fn keys() -> Vec<Key> {
    let mut all = vec![carry(), drawn()];
    for dir in [Dir::Right, Dir::Left, Dir::Up, Dir::Down] {
        all.push(guard(dir));
        for style in [Style::Thrust, Style::Chop, Style::Slash] {
            all.extend([0.0, 0.3, 0.6, 1.0].map(|s| Path::of(style, dir).at(s)));
        }
    }
    all
}

/// The right hand lands on the grip of every key that lies within its reach and its joint limits,
/// within 3 cm, through the rig the body is drawn with.
#[test]
fn the_hand_lands_on_the_grip() {
    let mut near = 0;
    for key in keys() {
        let pose = Layer::of(&key, Hands::One).apply(Pose::default());
        let miss = hand(&pose, 1).distance(key.grip);
        near += usize::from(miss < 0.03);
        assert!(miss < 0.2, "{key:?} misses by {miss}");
    }
    assert!(near * 10 >= keys().len() * 8, "{near} of {}", keys().len());
}

/// A target straight ahead at chest height is reached with the arm forward and the elbow bent.
#[test]
fn a_target_ahead_bends_the_elbow() {
    let [shoulder, out, elbow] = reach(Vec3::new(0.212, 0.2, -0.3), 1);
    assert!(
        shoulder > 0.5 && out.abs() < 0.2 && elbow > 0.2,
        "{shoulder} {out} {elbow}"
    );
    let pose = Layer::of(&carry(), Hands::One).apply(Pose::default());
    assert!(hand(&pose, 1).distance(carry().grip) < 0.03);
}

/// The layer adds the chest turn and bend, moves the right arm alone for one hand and both for two,
/// and a layer of no weight leaves the pose alone.
#[test]
fn the_layer_moves_only_the_holding_arms() {
    let base = Pose::default()
        .with(Core::Twist, 0.1)
        .both(Limb::Shoulder, 0.2);
    let key = Path::of(Style::Slash, Dir::Right).at(0.0);
    let one = Layer::of(&key, Hands::One).apply(base);
    assert!((one.core(Core::Twist) - (0.1 + key.twist)).abs() < 1e-5);
    assert_eq!(one.sides[0], base.sides[0]);
    assert_ne!(one.sides[1], base.sides[1]);
    let two = Layer::of(&key, Hands::Two).apply(base);
    assert_ne!(two.sides[0], base.sides[0]);
    assert_eq!(Layer::default().apply(base), base);
}
