//! Drawing primitives on a [`Build`]: oriented triangles and quads, boxes and bare colliders.

use bevy::prelude::*;

use super::brush::Mat;
use super::brush::{Build, Part, Shape};
use crate::world::shapes::srgb;

/// Box faces as unit-cube corners, outward direction and tone.
const FACES: [([[f32; 3]; 4], Vec3, f32); 6] = [
    (
        [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
        Vec3::NEG_Y,
        0.6,
    ),
    (
        [[0., 1., 0.], [1., 1., 0.], [1., 1., 1.], [0., 1., 1.]],
        Vec3::Y,
        1.08,
    ),
    (
        [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
        Vec3::Z,
        0.95,
    ),
    (
        [[0., 0., 0.], [1., 0., 0.], [1., 1., 0.], [0., 1., 0.]],
        Vec3::NEG_Z,
        0.82,
    ),
    (
        [[1., 0., 0.], [1., 0., 1.], [1., 1., 1.], [1., 1., 0.]],
        Vec3::X,
        0.9,
    ),
    (
        [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
        Vec3::NEG_X,
        0.76,
    ),
];

impl Build<'_> {
    /// Rotates a local direction into chunk space.
    pub fn dir(&self, v: Vec3) -> Vec3 {
        Quat::from_rotation_y(self.yaw) * v
    }

    /// Returns a stable pseudo-random value in `[0, 1)` for this structure and an index.
    pub fn roll(&self, i: u32) -> f32 {
        let h = (self.site.variant ^ i.wrapping_mul(0x9E37_79B9)).wrapping_mul(0x85EB_CA6B);
        let h = (h ^ (h >> 13)).wrapping_mul(0xC2B2_AE35);
        ((h >> 8) & 0xFFFF) as f32 / 65536.0
    }

    /// Returns a vertex color: the base color scaled by `tone` and a small per-quad variation.
    pub fn color(&mut self, rgb: [f32; 3], tone: f32) -> [f32; 4] {
        self.count = self.count.wrapping_add(1);
        let k = tone * (0.94 + self.roll(self.count) * 0.12);
        srgb(
            (rgb[0] * k).min(1.0),
            (rgb[1] * k).min(1.0),
            (rgb[2] * k).min(1.0),
        )
    }

    /// Adds a triangle wound so that its front faces the chunk-space direction `out`.
    pub fn tri_out(&mut self, p: [Vec3; 3], out: Vec3, color: [f32; 4]) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        match n.dot(out) >= 0.0 {
            true => self.mesh.tri(p[0], p[1], p[2], color),
            false => self.mesh.tri(p[0], p[2], p[1], color),
        };
    }

    /// Adds a quad wound so that its front faces the chunk-space direction `out`.
    pub fn quad_out(&mut self, p: [Vec3; 4], out: Vec3, color: [f32; 4]) {
        self.tri_out([p[0], p[1], p[2]], out, color);
        self.tri_out([p[0], p[2], p[3]], out, color);
    }

    /// Adds a collider box between two local corners without drawing it.
    pub fn collider(&mut self, lo: Vec3, hi: Vec3) {
        let rot = Quat::from_rotation_y(self.yaw);
        let pos = self.pt((lo + hi) * 0.5);
        self.parts.push(Part {
            pos,
            rot,
            shape: Shape::Cuboid(hi - lo),
        });
    }

    /// Adds a closed box between two local corners, with a collider when `mat` is solid.
    pub fn cuboid(&mut self, lo: Vec3, hi: Vec3, mat: Mat) {
        #[cfg(test)]
        let start = self.vertex_mark();
        for (corners, out, tone) in FACES {
            let p = corners.map(|c| self.pt(lo + (hi - lo) * Vec3::from_array(c)));
            let color = self.color(mat.rgb, tone);
            let out = self.dir(out);
            self.quad_out(p, out, color);
        }
        #[cfg(test)]
        self.note(super::audit::Piece::Closed, start);
        if mat.solid {
            self.collider(lo, hi);
        }
    }
}
