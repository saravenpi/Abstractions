//! Roofs on a [`Build`]: hipped, lean-to, flat with parapet and the style-aware cap.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::palette::SNOW;
use super::village::style::{Flavor, Style};

impl Build<'_> {
    /// Adds a hipped roof over the box `lo..hi` at eave height `lo.y`, ridge along local x.
    pub fn hip(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let (mid, half) = (
            (lo.z + hi.z) * 0.5,
            ((hi.z - lo.z) * 0.5).min((hi.x - lo.x) * 0.5),
        );
        let y = lo.y;
        let a = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, y, lo.z)));
        let b = [lo.x, hi.x].map(|x| self.pt(Vec3::new(x, y, hi.z)));
        let r = [lo.x + half, hi.x - half].map(|x| self.pt(Vec3::new(x, y + rise, mid)));
        let (top, under, end) = (
            self.color(mat.rgb, 1.05),
            self.color(mat.rgb, 0.65),
            self.color(mat.rgb, 0.85),
        );
        let front = self.dir(Vec3::new(0.0, half, rise));
        let back = self.dir(Vec3::new(0.0, half, -rise));
        self.sheet_quad([b[0], b[1], r[1], r[0]], front, [top, under]);
        self.sheet_quad([a[1], a[0], r[0], r[1]], back, [top, under]);
        let left = self.dir(Vec3::new(-rise, half, 0.0));
        let right = self.dir(Vec3::new(rise, half, 0.0));
        self.sheet_tri([a[0], b[0], r[0]], left, [end, under]);
        self.sheet_tri([a[1], b[1], r[1]], right, [end, under]);
    }

    /// Adds a single slope that is high at `lo.z` and falls to `lo.y` at `hi.z`.
    pub fn lean_to(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let (y, depth) = (lo.y, hi.z - lo.z);
        let hi_edge = |x: f32| self.pt(Vec3::new(x, y + rise, lo.z));
        let lo_edge = |x: f32| self.pt(Vec3::new(x, y, hi.z));
        let base = |x: f32| self.pt(Vec3::new(x, y, lo.z));
        let (a0, a1, b0, b1) = (hi_edge(lo.x), hi_edge(hi.x), lo_edge(lo.x), lo_edge(hi.x));
        let (c0, c1) = (base(lo.x), base(hi.x));
        let (top, under) = (self.color(mat.rgb, 1.05), self.color(mat.rgb, 0.65));
        let out = self.dir(Vec3::new(0.0, depth, rise));
        self.sheet_quad([a0, a1, b1, b0], out, [top, under]);
        let (left, right) = (self.dir(Vec3::NEG_X), self.dir(Vec3::X));
        self.sheet_tri([a0, b0, c0], left, [under, under]);
        self.sheet_tri([a1, b1, c1], right, [under, under]);
    }

    /// Adds a flat slab over `lo..hi` at height `lo.y` with a low parapet around it.
    pub fn flat_roof(&mut self, lo: Vec3, hi: Vec3, mat: Mat) {
        let (l, h) = (lo - Vec3::new(0.2, 0.0, 0.2), hi + Vec3::new(0.2, 0.0, 0.2));
        self.cuboid(l, Vec3::new(h.x, lo.y + 0.2, h.z), mat);
        let top = lo.y + 0.2;
        let ring = [
            ([l.x, top, l.z], [h.x, top + 0.45, l.z + 0.25]),
            ([l.x, top, h.z - 0.25], [h.x, top + 0.45, h.z]),
            ([l.x, top, l.z + 0.25], [l.x + 0.25, top + 0.45, h.z - 0.25]),
            ([h.x - 0.25, top, l.z + 0.25], [h.x, top + 0.45, h.z - 0.25]),
        ];
        self.boxes(&ring, Mat::deco(mat.rgb.map(|c| c * 0.85)));
    }

    /// Adds the roof the village style asks for over the box `(lo, hi)`; `pick` in `[0, 1)` chooses the shape.
    pub fn cap_roof(&mut self, st: &Style, span: (Vec3, Vec3), rise: f32, pick: f32) {
        let (lo, hi) = span;
        let mat = Mat::deco(st.roof[0]);
        match st.flavor {
            Flavor::Adobe => self.flat_roof(lo, hi, mat),
            Flavor::Stilt => {
                let wide = Vec3::new(0.5, 0.0, 0.5);
                self.hip(lo - wide, hi + wide, rise * 1.2, mat);
            }
            Flavor::Cherry => self.tiers(lo, hi, rise, mat),
            Flavor::Taiga | Flavor::Timber => self.pitched(st.flavor, span, rise, (pick, mat)),
        }
    }

    /// Adds a two-tier hipped roof with wide eaves.
    fn tiers(&mut self, lo: Vec3, hi: Vec3, rise: f32, mat: Mat) {
        let wide = Vec3::new(0.7, 0.0, 0.7);
        self.hip(lo - wide, hi + wide, rise * 0.7, mat);
        let inset = Vec3::new(1.0, 0.9, 0.7);
        self.hip(lo + inset, hi - inset + Vec3::Y * 1.8, rise * 0.8, mat);
    }

    /// Adds a gable or hip roof, with a snow layer on top in taiga villages.
    fn pitched(&mut self, flavor: Flavor, span: (Vec3, Vec3), rise: f32, shape: (f32, Mat)) {
        let (lo, hi) = span;
        let (pick, mat) = shape;
        let lift = Vec3::Y * 0.12;
        let snow = Mat::deco(SNOW);
        let gable = pick < 0.65;
        let layers = if flavor == Flavor::Taiga {
            vec![(Vec3::ZERO, mat), (lift, snow)]
        } else {
            vec![(Vec3::ZERO, mat)]
        };
        for (up, m) in layers {
            match gable {
                true => self.gable(lo + up, hi + up, rise, m),
                false => self.hip(lo + up, hi + up, rise, m),
            }
        }
    }
}
