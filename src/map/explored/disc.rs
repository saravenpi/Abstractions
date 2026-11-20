//! The leaves under a disc of ground.

use std::f64::consts::FRAC_PI_4;

use bevy::math::DVec3;

use super::grid::{cell, direction, leaf_of};
use super::morton::along;

/// Calls `each` with the leaf under unit direction `dir` and every leaf at `depth` whose centre
/// lies within `angle` radians of it. A leaf may come twice.
pub fn disc(dir: DVec3, angle: f64, depth: u64, mut each: impl FnMut(u64)) {
    let home = leaf_of(dir, depth);
    each(home);
    let (face, _, (x, y)) = cell(home);
    let n = along(depth);
    let steps = (angle * n / FRAC_PI_4 / 2.0 * 1.8).ceil().min(n) as i64 + 1;
    let near = angle.cos();
    for dy in -steps..=steps {
        for dx in -steps..=steps {
            let at = direction(face, depth, (x + dx, y + dy));
            if at.dot(dir) >= near {
                each(leaf_of(at, depth));
            }
        }
    }
}
