//! Thin pieces shared by the pond plants: ribbons, pads and flowers.

use std::f32::consts::TAU;

use bevy::prelude::*;

use crate::world::shapes::{FlatMesh, shade};
use crate::world::solids::fan;

/// Height of the floating parts above the water level, which keeps them clear of the sea plane.
pub const FLOAT: f32 = 0.02;

/// Adds a triangle visible from both sides.
pub fn both(m: &mut FlatMesh, tri: [Vec3; 3], color: [f32; 4]) {
    m.tri(tri[0], tri[1], tri[2], color);
    m.tri(tri[2], tri[1], tri[0], color);
}

/// Adds a thin vertical ribbon from `from` to `to`, turned by `yaw`.
pub fn stalk(m: &mut FlatMesh, ends: [Vec3; 2], yaw: f32, color: [f32; 4]) {
    let s = Vec3::new(yaw.cos(), 0.0, yaw.sin()) * 0.03;
    let [from, to] = ends;
    both(m, [from - s, from + s, to - s], color);
    both(m, [from + s, to + s, to - s], color);
}

/// Adds a round pad with a notch, from its radius and the height its rim is raised by.
pub fn pad(m: &mut FlatMesh, center: Vec3, size: [f32; 2], color: [f32; 4]) {
    let ring = |i: u32| {
        let a = i as f32 / 9.0 * TAU;
        center + Vec3::new(a.cos() * size[0], size[1], a.sin() * size[0])
    };
    for i in 1..9 {
        both(m, [center, ring(i + 1), ring(i)], color);
    }
}

/// Adds a flower of upright petals, white or pink.
pub fn bloom(m: &mut FlatMesh, at: Vec3, yaw: f32, pink: bool) {
    let color = if pink {
        [0.97, 0.55, 0.72]
    } else {
        [0.97, 0.97, 0.94]
    };
    fan(m, at, [8.0, 0.13, 0.2, 0.0, 0.12], yaw, shade(color, 1.0));
}
