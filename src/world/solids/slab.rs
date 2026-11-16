//! A rotated box for planks, chests and crates.

use bevy::prelude::*;

use super::varied;
use crate::world::shapes::FlatMesh;

/// Adds a box of half sizes `half` centered on `center`, rolled around the `x` axis by `roll` radians.
pub fn slab(mesh: &mut FlatMesh, center: Vec3, half: Vec3, roll: f32, color: [f32; 4]) {
    let rot = Quat::from_rotation_x(roll);
    for axis in 0..3 {
        for (n, sign) in [-1.0_f32, 1.0].into_iter().enumerate() {
            let at = |a: f32, b: f32| {
                let mut s = [0.0; 3];
                s[axis] = sign;
                s[(axis + 1) % 3] = a;
                s[(axis + 2) % 3] = b;
                center + rot * (half * Vec3::from(s))
            };
            let q = [at(-1.0, -1.0), at(1.0, -1.0), at(1.0, 1.0), at(-1.0, 1.0)];
            let o = if sign > 0.0 {
                [0, 1, 2, 3]
            } else {
                [0, 3, 2, 1]
            };
            let tone = varied(color, axis * 2 + n);
            mesh.tri(q[o[0]], q[o[1]], q[o[2]], tone);
            mesh.tri(q[o[0]], q[o[2]], q[o[3]], tone);
        }
    }
}
