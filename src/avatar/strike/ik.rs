//! Reach: two-bone inverse kinematics that put a hand on the grip.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::super::pose::{LIMB_LIMITS, Limb};
use super::super::tune::{FOREARM, SHOULDER, UPPER_ARM};
use super::tune::HAND_GRIP;

/// Returns the shoulder swing, the arm lift and the elbow bend, in the order of the pose joints,
/// that put the middle of the hand of `side` (0 left, 1 right) on `target` in the torso frame.
///
/// The rig swings the whole arm in one plane through the shoulder that holds the forward axis and
/// the arm's lift, and the elbow bends only one way in it: the forearm folds toward the front of
/// the upper arm, as a person's does. Two lifts put that plane through the target; for each the
/// upper arm is swung so the bent arm reaches the target, kept inside the limits of the shoulder,
/// and the forearm aims from the elbow at the target, inside the limits of the elbow. Of the two,
/// the one whose hand lands nearer the target wins. Every angle returned is inside its limits, so
/// nothing clamps the arm afterwards into a pose that misses the grip.
pub fn reach(target: Vec3, side: usize) -> [f32; 3] {
    let sign = [-1.0, 1.0][side];
    let shoulder = Vec3::new(SHOULDER[0] * sign, SHOULDER[1], SHOULDER[2]);
    let t = target - shoulder;
    let t = Vec3::new(sign * t.x, t.y, t.z);
    let (low, high) = LIMB_LIMITS[Limb::ArmOut as usize];
    [t.x.atan2(-t.y), (-t.x).atan2(t.y)]
        .map(|out| solve(t, out.clamp(low, high)))
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map_or([0.0; 3], |(joints, _)| joints)
}

/// Returns the joints that put the hand nearest `t`, a target from the shoulder with +X outward,
/// with the arm lifted `out`, and how far the hand then misses the target.
fn solve(t: Vec3, out: f32) -> ([f32; 3], f32) {
    let (a, b) = (UPPER_ARM, FOREARM + HAND_GRIP);
    let along = t.x * out.sin() - t.y * out.cos();
    let (forward, off) = (-t.z, t.x * out.cos() + t.y * out.sin());
    let length = along
        .hypot(forward)
        .clamp((a - b).abs() + 1e-3, a + b - 1e-3);
    let cos = ((a * a + b * b - length * length) / (2.0 * a * b)).clamp(-1.0, 1.0);
    let bent = PI - cos.acos();
    let rest = (b * bent.sin()).atan2(a + b * bent.cos());
    let swing = swing(forward.atan2(along) - rest);
    let elbow = Vec2::new(swing.cos(), swing.sin()) * a;
    let to = Vec2::new(along, forward) - elbow;
    let (low, high) = LIMB_LIMITS[Limb::Elbow as usize];
    let bend = wrap(to.y.atan2(to.x) - swing).clamp(low, high);
    let hand = elbow + Vec2::new((swing + bend).cos(), (swing + bend).sin()) * b;
    let miss = hand.distance(Vec2::new(along, forward)).hypot(off);
    ([swing, out, bend], miss)
}

/// Returns `angle` turned by whole circles into the limits of the shoulder swing, or onto the limit
/// nearer it around the circle when no turn brings it inside.
fn swing(angle: f32) -> f32 {
    let (low, high) = LIMB_LIMITS[Limb::Shoulder as usize];
    let turned = low + (angle - low).rem_euclid(TAU);
    match turned <= high {
        true => turned,
        false if turned - high < low + TAU - turned => high,
        false => low,
    }
}

/// Returns `angle` turned by whole circles into a half turn either way.
fn wrap(angle: f32) -> f32 {
    (angle + PI).rem_euclid(TAU) - PI
}
