//! Composite pieces on a [`Build`]: foundations, ramps, roofs, pillars, cones and rubble.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::brush::{Build, Mat, Part, Shape};
use super::palette::STONE;
use crate::world::solids::{cone, icosa, prism};

/// Steepest walkable ramp slope, rise over run.
const RAMP_SLOPE: f32 = 0.22;

impl Build<'_> {
    /// Adds a box foundation centered on the frame, top at `y = 0`, reaching the site depth below.
    pub fn foundation(&mut self, half: Vec2, mat: Mat) {
        let depth = self.site.depth;
        let (lo, hi) = (
            Vec3::new(-half.x, -depth, -half.y),
            Vec3::new(half.x, 0.0, half.y),
        );
        self.cuboid(lo, hi, mat);
    }

    /// Adds a round foundation that rises `lift` above `y = 0`, with a flat top.
    pub fn round_foundation(&mut self, r: f32, sides: u32, lift: f32, mat: Mat) {
        let depth = self.site.depth;
        self.pillar(Vec3::Y * -depth, Vec2::new(r, depth + lift), sides, mat);
        self.spire(Vec3::Y * lift, Vec2::new(r, 0.001), sides, mat.rgb);
    }

    /// Adds an upright prism with its bottom circle centered on the local point `base`.
    pub fn pillar(&mut self, base: Vec3, size: Vec2, sides: u32, mat: Mat) {
        let color = self.color(mat.rgb, 1.0);
        let at = self.pt(base);
        prism(&mut self.mesh, at, size, sides, color);
        if mat.solid {
            let pos = self.pt(base + Vec3::Y * size.y * 0.5);
            self.parts.push(Part {
                pos,
                rot: Quat::IDENTITY,
                shape: Shape::Cylinder(size.x, size.y),
            });
        }
    }

    /// Adds a cone with its base circle centered on the local point `base`.
    pub fn spire(&mut self, base: Vec3, size: Vec2, sides: u32, rgb: [f32; 3]) {
        let color = self.color(rgb, 1.0);
        let at = self.pt(base);
        cone(&mut self.mesh, at, size, sides, color);
    }

    /// Adds a boulder on the ground at the local point `(x, z)`.
    pub fn rock(&mut self, x: f32, z: f32, size: f32, seed: u32) {
        let y = self.ground_at(x, z) - self.base_y + size * 0.3;
        let at = self.pt(Vec3::new(x, y, z));
        let color = self.color(STONE, 1.0);
        icosa(
            &mut self.mesh,
            at,
            Vec3::new(size, size * 0.65, size * 0.85),
            (0.25, seed),
            color,
        );
    }

    /// Lowers the frame to the highest ground within `r` of it, plus `lift`.
    pub fn settle(&mut self, r: f32, lift: f32) {
        let ring = (0..8).map(|i| {
            let a = i as f32 / 8.0 * TAU;
            self.ground_at(a.sin() * r, a.cos() * r)
        });
        let top = ring.fold(self.ground_at(0.0, 0.0), f32::max);
        self.base_y = top + lift;
    }

    /// Adds a gabled roof with its ridge along local x. Eaves sit at `lo.y`, between `lo.z` and `hi.z`.
    pub fn gable(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let (mid, half) = ((lo.z + hi.z) * 0.5, (hi.z - lo.z) * 0.5);
        let a = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, lo.z)));
        let b = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, hi.z)));
        let r = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y + rise, mid)));
        let (front, back) = (
            self.dir(Vec3::new(0.0, half, rise)),
            self.dir(Vec3::new(0.0, half, -rise)),
        );
        let (top, under, end) = (
            self.color(mat.rgb, 1.05),
            self.color(mat.rgb, 0.65),
            self.color(mat.rgb, 0.85),
        );
        for (quad, out) in [
            ([b[0], b[1], r[1], r[0]], front),
            ([a[1], a[0], r[0], r[1]], back),
        ] {
            self.sheet_quad(quad, out, [top, under]);
        }
        for (i, out) in [Vec3::NEG_X, Vec3::X].into_iter().enumerate() {
            let out = self.dir(out);
            self.sheet_tri([a[i], b[i], r[i]], out, [end, under]);
        }
    }

    /// Adds a ramp from the local point `at`, heading along local +z down to the ground.
    pub fn ramp(&mut self, at: Vec3, width: f32) {
        let top = self.base_y + at.y;
        let drop_at = |s: &Self, len: f32| (top - s.ground_at(at.x, at.z + len)).max(0.25);
        let mut len = 3.0;
        for _ in 0..3 {
            len = (drop_at(self, len) / RAMP_SLOPE).max(1.5);
        }
        let drop = drop_at(self, len);
        let w = width * 0.5;
        let corner =
            |s: &Self, x: f32, down: f32, z: f32| s.pt(Vec3::new(at.x + x, at.y - down, at.z + z));
        let (a, b) = (corner(self, -w, 0.0, 0.0), corner(self, w, 0.0, 0.0));
        let (c, d) = (corner(self, w, drop, len), corner(self, -w, drop, len));
        let (al, bl) = (corner(self, -w, drop, 0.0), corner(self, w, drop, 0.0));
        let hyp = len.hypot(drop);
        let up = self.dir(Vec3::new(0.0, len, drop) / hyp);
        let (top_c, side_c) = (self.color(STONE, 1.0), self.color(STONE, 0.8));
        self.sheet_quad([a, b, c, d], up, [top_c, side_c]);
        self.sheet_tri([a, al, d], self.dir(Vec3::NEG_X), [side_c, side_c]);
        self.sheet_tri([b, bl, c], self.dir(Vec3::X), [side_c, side_c]);
        let tilt = Quat::from_rotation_y(self.yaw)
            * Quat::from_rotation_arc(Vec3::Z, Vec3::new(0.0, -drop, len) / hyp);
        let mid = self.pt(Vec3::new(at.x, at.y - drop * 0.5, at.z + len * 0.5)) - up * 0.25;
        self.parts.push(Part {
            pos: mid,
            rot: tilt,
            shape: Shape::Cuboid(Vec3::new(width, 0.5, hyp)),
        });
    }
}
