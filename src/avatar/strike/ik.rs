//! Reach: two-bone inverse kinematics that put a hand on the grip.

use std::f32::consts::PI;

use bevy::prelude::*;

use super::super::tune::{FOREARM, SHOULDER, UPPER_ARM};
use super::tune::HAND_GRIP;

/// Returns the shoulder swing, the arm lift and the elbow bend, in the order of the pose joints,
/// that put the middle of the hand of `side` (0 left, 1 right) on `target` in the torso frame.
///
/// The elbow follows from the law of cosines over the upper arm and the forearm with the hand; the
/// lift turns the arm into the vertical plane through the target, and the swing turns it forward in
/// that plane, less the angle the bent elbow already puts between the upper arm and the target. A
/// target out of reach is reached towards, the arm straight.
pub fn reach(target: Vec3, side: usize) -> [f32; 3] {
    let sign = [-1.0, 1.0][side];
    let shoulder = Vec3::new(SHOULDER[0] * sign, SHOULDER[1], SHOULDER[2]);
    let t = target - shoulder;
    let (a, b) = (UPPER_ARM, FOREARM + HAND_GRIP);
    let length = t.length().clamp((a - b).abs() + 1e-3, a + b - 1e-3);
    let cos = ((a * a + b * b - length * length) / (2.0 * a * b)).clamp(-1.0, 1.0);
    let elbow = PI - cos.acos();
    let rest = (b * elbow.sin()).atan2(a + b * elbow.cos());
    let x = sign * t.x;
    let wide = x.hypot(t.y);
    let (out, down) = match t.y <= 0.0 {
        true => (x.atan2(-t.y), -wide),
        false => ((-x).atan2(t.y), wide),
    };
    let swing = (-t.z).atan2(-down);
    [swing - rest, out, elbow]
}
