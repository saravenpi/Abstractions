//! Colliders of roofs: the convex body of each gabled or hipped roof, so that whoever lands on a roof stands
//! on exactly what they see, and a thin box just under each single slope.

use bevy::prelude::*;

use super::brush::{Build, Part, Shape};

/// Thickness of the box under a single roof slope, in meters: thin enough to stay out of the room under the
/// lowest pitch, the shed.
pub const THICK: f32 = 0.1;

impl Build<'_> {
    /// Adds a box whose top face is the local rectangle on the edge `from..to` that rises by `up`, which is
    /// square to the edge, and that reaches [`THICK`] under it.
    pub fn plate(&mut self, from: Vec3, to: Vec3, up: Vec3) {
        let along = to - from;
        let rise = up.normalize();
        let mut x = along.normalize();
        let mut n = rise.cross(x);
        if n.y < 0.0 {
            (x, n) = (-x, -n);
        }
        let local = Quat::from_mat3(&Mat3::from_cols(x, n, rise));
        let center = from + along * 0.5 + up * 0.5 - n * THICK * 0.5;
        self.parts.push(Part {
            pos: self.pt(center),
            rot: Quat::from_rotation_y(self.yaw) * local,
            shape: Shape::Cuboid(Vec3::new(along.length(), THICK, up.length())),
        });
    }

    /// Adds the collider of a roof over `lo..hi` whose eaves lie at `lo.y` and whose ridge runs along local
    /// x, `rise` above them, and stops `inset` short of both ends: nothing for a gabled roof, half the depth
    /// for a hipped one. It is the convex body the drawn slopes enclose, from the eaves up.
    pub fn roof_body(&mut self, lo: Vec3, hi: Vec3, rise: f32, inset: f32) {
        let (mid, half) = ((lo + hi) * 0.5, (hi - lo) * 0.5);
        let ridge = half.x - inset;
        let corner = |x: f32, z: f32| Vec3::new(x * half.x, 0.0, z * half.z);
        let points = [
            corner(-1.0, -1.0),
            corner(1.0, -1.0),
            corner(1.0, 1.0),
            corner(-1.0, 1.0),
            Vec3::new(-ridge, rise, 0.0),
            Vec3::new(ridge, rise, 0.0),
        ];
        self.parts.push(Part {
            pos: self.pt(Vec3::new(mid.x, lo.y, mid.z)),
            rot: Quat::from_rotation_y(self.yaw),
            shape: Shape::Hull(points),
        });
    }
}
