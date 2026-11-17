//! Roof shapes on a [`Build`]: gabled, hipped, lean-to, flat with a parapet and striped awnings.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::tags::Tag;

/// Returns the height at depth `z` of the single slope over `lo..hi`: `lo.y + rise` at `lo.z`, `lo.y` at `hi.z`.
pub fn lean_height(lo: Vec3, hi: Vec3, rise: f32, z: f32) -> f32 {
    lo.y + rise * (hi.z - z) / (hi.z - lo.z)
}

impl Build<'_> {
    /// Adds the two slopes of a gabled roof with its ridge along local x and open ends. Eaves sit at `lo.y`,
    /// between `lo.z` and `hi.z`.
    pub fn slopes(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let (mid, half) = ((lo.z + hi.z) * 0.5, (hi.z - lo.z) * 0.5);
        let a = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, lo.z)));
        let b = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, hi.z)));
        let r = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y + rise, mid)));
        let front = self.dir(Vec3::new(0.0, half, rise));
        let back = self.dir(Vec3::new(0.0, half, -rise));
        let colors = [self.color(mat.rgb, 1.05), self.color(mat.rgb, 0.65)];
        self.tagged(Tag::Roof, |s| {
            s.sheet_quad([b[0], b[1], r[1], r[0]], front, colors);
            s.sheet_quad([a[1], a[0], r[0], r[1]], back, colors);
        });
    }

    /// Adds a gabled roof with its ridge along local x and both ends closed by a triangle.
    pub fn gable(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        self.slopes(lo, hi, rise, mat);
        let mid = (lo.z + hi.z) * 0.5;
        let colors = [self.color(mat.rgb, 0.85), self.color(mat.rgb, 0.65)];
        for (x, out) in [(lo.x, Vec3::NEG_X), (hi.x, Vec3::X)] {
            let corners = [(lo.y, lo.z), (lo.y, hi.z), (lo.y + rise, mid)];
            let tri = corners.map(|(y, z)| self.pt(Vec3::new(x, y, z)));
            let out = self.dir(out);
            self.sheet_tri(tri, out, colors);
        }
    }

    /// Adds a hipped roof over the box `lo..hi` at eave height `lo.y`, ridge along local x.
    pub fn hip(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let mid = (lo.z + hi.z) * 0.5;
        let half = ((hi.z - lo.z) * 0.5).min((hi.x - lo.x) * 0.5);
        let a = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, lo.z)));
        let b = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, lo.y, hi.z)));
        let r = [lo.x + half, hi.x - half].map(|x| self.pt(Vec3::new(x, lo.y + rise, mid)));
        let slope = [self.color(mat.rgb, 1.05), self.color(mat.rgb, 0.65)];
        let end = [self.color(mat.rgb, 0.85), slope[1]];
        let out = [(0.0, rise), (0.0, -rise), (-rise, 0.0), (rise, 0.0)];
        let [front, back, left, right] = out.map(|(x, z)| self.dir(Vec3::new(x, half, z)));
        self.tagged(Tag::Roof, |s| {
            s.sheet_quad([b[0], b[1], r[1], r[0]], front, slope);
            s.sheet_quad([a[1], a[0], r[0], r[1]], back, slope);
            s.sheet_tri([a[0], b[0], r[0]], left, end);
            s.sheet_tri([a[1], b[1], r[1]], right, end);
        });
    }

    /// Adds a single slope that is high at `lo.z` and falls to `lo.y` at `hi.z`, with open sides.
    pub fn slope(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let corner = |x: f32, y: f32, z: f32| self.pt(Vec3::new(x, y, z));
        let top = [
            corner(lo.x, lo.y + rise, lo.z),
            corner(hi.x, lo.y + rise, lo.z),
        ];
        let low = [corner(hi.x, lo.y, hi.z), corner(lo.x, lo.y, hi.z)];
        let colors = [self.color(mat.rgb, 1.05), self.color(mat.rgb, 0.65)];
        let out = self.dir(Vec3::new(0.0, hi.z - lo.z, rise));
        self.tagged(Tag::Roof, |s| {
            s.sheet_quad([top[0], top[1], low[0], low[1]], out, colors)
        });
    }

    /// Adds a single slope that is high at `lo.z` and falls to `lo.y` at `hi.z`, its two sides closed down to `lo.y`.
    pub fn lean_to(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        self.slope(lo, hi, rise, mat);
        let under = self.color(mat.rgb, 0.65);
        for (x, out) in [(lo.x, Vec3::NEG_X), (hi.x, Vec3::X)] {
            let corners = [(lo.y + rise, lo.z), (lo.y, hi.z), (lo.y, lo.z)];
            let tri = corners.map(|(y, z)| self.pt(Vec3::new(x, y, z)));
            let out = self.dir(out);
            self.sheet_tri(tri, out, [under, under]);
        }
    }

    /// Adds a sloping awning over `lo..hi` cut along x into one strip per color of `stripes`, its two outer
    /// sides closed down to `lo.y`.
    pub fn awning(&mut self, lo: Vec3, hi: Vec3, rise: f32, stripes: &[[f32; 3]]) {
        let width = (hi.x - lo.x) / stripes.len() as f32;
        for (i, rgb) in stripes.iter().enumerate() {
            let x = lo.x + width * i as f32;
            let (from, to) = (Vec3::new(x, lo.y, lo.z), Vec3::new(x + width, lo.y, hi.z));
            self.slope(from, to, rise, Mat::deco(*rgb));
        }
        let ends = [(lo.x, Vec3::NEG_X, 0), (hi.x, Vec3::X, stripes.len() - 1)];
        for (x, out, stripe) in ends {
            let under = self.color(stripes[stripe], 0.65);
            let corners = [(lo.y + rise, lo.z), (lo.y, hi.z), (lo.y, lo.z)];
            let tri = corners.map(|(y, z)| self.pt(Vec3::new(x, y, z)));
            let out = self.dir(out);
            self.sheet_tri(tri, out, [under, under]);
        }
    }

    /// Adds a flat slab over `lo..hi` at height `lo.y` with a low parapet around it.
    pub fn flat_roof(&mut self, lo: Vec3, hi: Vec3, mat: Mat) {
        let (l, h) = (lo - Vec3::new(0.2, 0.0, 0.2), hi + Vec3::new(0.2, 0.0, 0.2));
        let top = lo.y + 0.2;
        let ring = [
            ([l.x, top, l.z], [h.x, top + 0.45, l.z + 0.25]),
            ([l.x, top, h.z - 0.25], [h.x, top + 0.45, h.z]),
            ([l.x, top, l.z + 0.25], [l.x + 0.25, top + 0.45, h.z - 0.25]),
            ([h.x - 0.25, top, l.z + 0.25], [h.x, top + 0.45, h.z - 0.25]),
        ];
        self.tagged(Tag::Roof, |s| {
            s.cuboid(l, Vec3::new(h.x, top, h.z), mat);
            s.boxes(&ring, Mat::deco(mat.rgb.map(|c| c * 0.85)));
        });
    }
}
