//! Chains of joints: follow-the-leader distance constraints with a bend limit, and spring lag.
//!
//! A chain is its joints from root to tip, one more than its segments. Each segment keeps its
//! length exactly and bends from the one before by at most the limit. The first segment bends from
//! a base direction, the axis of the part the chain hangs from.

use bevy::math::{Quat, Vec3};

use super::ik::unit;
use super::spring::Spring;

/// Returns `want` turned toward `prev` as far as needed to lie within `limit` radians of it.
pub fn bend(prev: Vec3, want: Vec3, limit: f32) -> Vec3 {
    let angle = prev.angle_between(want);
    if angle <= limit || !angle.is_finite() {
        return want;
    }
    let axis = unit(prev.cross(want), prev.any_orthonormal_vector());
    Quat::from_axis_angle(axis, limit) * prev
}

/// Pulls `points` after their first joint, set to `anchor`: each joint keeps its distance from the
/// one before and bends by at most `limit` from it, the first segment from `base`.
pub fn follow(points: &mut [Vec3], lengths: &[f32], (anchor, base): (Vec3, Vec3), limit: f32) {
    let Some(first) = points.first_mut() else {
        return;
    };
    *first = anchor;
    let mut prev = unit(base, Vec3::NEG_Y);
    for (i, &length) in lengths.iter().enumerate().take(points.len() - 1) {
        let want = unit(points[i + 1] - points[i], prev);
        let dir = bend(prev, want, limit);
        points[i + 1] = points[i] + dir * length;
        prev = dir;
    }
}

/// Moves every joint of `points` after the first toward its place in `targets` through `spring`,
/// with the velocities `vels`, by `dt` seconds: the chain lags behind the body that carries it.
pub fn lag(points: &mut [Vec3], vels: &mut [Vec3], targets: &[Vec3], spring: &Spring, dt: f32) {
    for ((at, vel), &target) in points.iter_mut().zip(vels.iter_mut()).zip(targets).skip(1) {
        spring.drive((at, vel), target, Vec3::ZERO, dt);
    }
}

/// Returns the joints of a straight chain from `anchor` along `dir` with segments `lengths`.
pub fn straight(anchor: Vec3, dir: Vec3, lengths: &[f32]) -> Vec<Vec3> {
    let dir = unit(dir, Vec3::NEG_Y);
    let mut points = vec![anchor];
    let mut at = anchor;
    for &length in lengths {
        at += dir * length;
        points.push(at);
    }
    points
}
