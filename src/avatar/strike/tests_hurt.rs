//! Tests of what hurt arms and a hurt torso leave of a strike.

use super::act::Timing;
use super::dir::Dir;
use super::hurt::{Hurt, steadied};
use super::key::carry;
use super::layer::Layer;
use super::path::Path;
use super::stage::blade;
use crate::condition::{Condition, Part};
use crate::craft::build::Hands;
use bevy::prelude::*;

/// A whole body leaves the swing as it is; a hurt arm and a hurt torso slow it.
#[test]
fn hurt_arms_and_torso_slow_a_swing() {
    let held = blade();
    let path = Path::of(held.style, Dir::Right);
    let whole = Timing::of(&held, &path);
    assert_eq!(whole.felt(&Hurt::default()), whole);
    let mut body = Condition::default();
    body.hurt(Part::ArmR, 300.0, true);
    body.hurt(Part::Torso, 1200.0, true);
    let slowed = whole.felt(&Hurt::of(&body, Hands::One));
    assert!(slowed.swing > whole.swing && slowed.recover > whole.recover);
}

/// A broken arm drops a two-handed tool to one hand, whose wind-up is longer, and the layer lets
/// the broken arm hang.
#[test]
fn a_broken_arm_drops_to_one_hand() {
    let mut body = Condition::default();
    body.hurt(Part::ArmL, 1.0e6, true);
    let hurt = Hurt::of(&body, Hands::Two);
    assert!(hurt.short && hurt.usable == [false, true]);
    assert!(Hurt::of(&body, Hands::One).grip() == 1.0 && hurt.grip() > 1.0);
    let key = carry();
    let layer = Layer::held(&key, Hands::Two, hurt.usable);
    assert!(layer.arms[0].is_none() && layer.arms[1].is_some());
    assert!(Layer::of(&key, Hands::Two).arms[0].is_some());
}

/// An arm that cannot hold the aim steady turns it; a steady one leaves it alone.
#[test]
fn a_weak_arm_shakes_the_aim() {
    let aim = Vec3::NEG_Z;
    assert!((steadied(aim, 1.0, 7) - aim).length() < 1e-6);
    assert!((steadied(aim, 0.0, 7) - aim).length() > 1e-3);
}
