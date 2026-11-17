//! Comfort: a small head bob that can be switched off, a landing dip, and a level horizon on the ground.

use bevy::prelude::*;

use super::world::Boxes;
use super::{moment, run};
use crate::view::tune::{BOB, DIP};
use crate::view::{Kind, Moment, Rig, ViewPrefs};

/// Returns the lowest and highest eye height over two seconds of walking, and the largest roll.
fn walk(body: &mut Moment) -> (f32, f32, f32) {
    let (mut rig, world) = (Rig::default(), Boxes::floor());
    body.target.velocity = Vec3::new(1.0, 0.0, -5.0);
    let (mut low, mut high, mut roll) = (f32::MAX, f32::MIN, 0.0_f32);
    for _ in 0..120 {
        let pose = run(&mut rig, body, &world, 1).pose;
        low = low.min(pose.position.y);
        high = high.max(pose.position.y);
        roll = roll.max((pose.rotation * Vec3::X).y.abs());
    }
    (low, high, roll)
}

/// The first person camera bobs by a few centimetres with each step and never rolls on the
/// ground, even when the body reports a bank. With the option off the eyes stay level.
#[test]
fn head_bob_is_small_and_optional() {
    let mut body = moment(Kind::OnFoot, ViewPrefs::default());
    body.target.bank = 0.5;
    let (low, high, roll) = walk(&mut body);
    assert!(high - low > BOB && high - low <= 2.0 * BOB + 1.0e-4);
    assert!((0.5 * (high + low) - 1.6).abs() < 0.01 && roll < 1.0e-6);
    body.prefs.head_bob = false;
    let (low, high, _) = walk(&mut body);
    assert!(high - low < 1.0e-5 && (low - 1.6).abs() < 1.0e-5);
}

/// A landing dips the first person camera by a few centimetres, at most the limit, and the
/// camera comes back up without rising above the eyes.
#[test]
fn landing_dips_and_recovers() {
    let (mut rig, world) = (Rig::default(), Boxes::default());
    let mut body = moment(Kind::OnFoot, ViewPrefs::default());
    body.prefs.head_bob = false;
    body.target.velocity = Vec3::NEG_Y * 9.0;
    run(&mut rig, &mut body, &world, 30);
    body.target.velocity = Vec3::ZERO;
    let mut deepest = 0.0_f32;
    for _ in 0..90 {
        let shown = run(&mut rig, &mut body, &world, 1);
        let below = body.target.position.y + 0.7 - shown.pose.position.y;
        assert!(below > -1.0e-4, "the camera rose above the eyes by {below}");
        deepest = deepest.max(below);
    }
    let end = run(&mut rig, &mut body, &world, 1).pose.position.y;
    assert!(
        deepest > 0.05 && deepest <= DIP.1 + 1.0e-3,
        "the dip is {deepest} m"
    );
    assert!((end - body.target.position.y - 0.7).abs() < 2.0e-3);
}
