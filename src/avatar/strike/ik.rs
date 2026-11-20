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
/// the arm's lift, so the lift is chosen to put that plane through the target, inside the joint
/// limits. The elbow follows from the law of cosines over the upper arm and the forearm with the
/// hand; the swing turns the arm forward in the plane, less the angle the bent elbow already puts
/// between the upper arm and the target. A target out of reach, or off the plane the limits allow,
/// is reached towards.
pub fn reach(target: Vec3, side: usize) -> [f32; 3] {
    let sign = [-1.0, 1.0][side];
    let shoulder = Vec3::new(SHOULDER[0] * sign, SHOULDER[1], SHOULDER[2]);
    let t = target - shoulder;
    let (out, along) = plane(sign * t.x, t.y);
    let forward = -t.z;
    let (a, b) = (UPPER_ARM, FOREARM + HAND_GRIP);
    let length = along
        .hypot(forward)
        .clamp((a - b).abs() + 1e-3, a + b - 1e-3);
    let cos = ((a * a + b * b - length * length) / (2.0 * a * b)).clamp(-1.0, 1.0);
    let elbow = PI - cos.acos();
    let rest = (b * elbow.sin()).atan2(a + b * elbow.cos());
    [swing(forward.atan2(along) - rest), out, elbow]
}

/// Returns the lift of the arm and the distance of the target along the arm hanging at that lift,
/// for a target `x` outward and `y` up from the shoulder: the lift of the plane that holds the
/// target, or the nearer of the two limits when none of the two planes that could does.
fn plane(x: f32, y: f32) -> (f32, f32) {
    let (low, high) = LIMB_LIMITS[Limb::ArmOut as usize];
    let miss = |out: f32| (out, x * out.cos() + y * out.sin());
    let (out, _) = [x.atan2(-y), (-x).atan2(y)]
        .map(|out| miss(out.clamp(low, high)))
        .into_iter()
        .min_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap_or((0.0, 0.0));
    (out, x * out.sin() - y * out.cos())
}

/// Returns `angle` turned a full circle if that brings it inside the limits of the shoulder swing.
fn swing(angle: f32) -> f32 {
    let (low, high) = LIMB_LIMITS[Limb::Shoulder as usize];
    if angle < low && angle + TAU <= high {
        angle + TAU
    } else {
        angle
    }
}
