//! Flat pieces on a [`Build`]: panes, double-sided sheets and frame placement on the ground.

use bevy::prelude::*;

use super::brush::{Build, Mat};
use super::site::Site;

impl Build<'_> {
    /// Adds a one-sided quad from four local corners, facing the local direction `out`.
    pub fn pane(&mut self, corners: [Vec3; 4], out: Vec3, rgb: [f32; 3]) {
        let color = self.color(rgb, 1.0);
        let out = self.dir(out);
        self.quad_out(corners.map(|c| self.pt(c)), out, color);
    }

    /// Adds a horizontal one-sided quad at local height `y` between two local corners `(x, z)`.
    pub fn floor_pane(&mut self, lo: Vec2, hi: Vec2, y: f32, rgb: [f32; 3]) {
        let c = |x: f32, z: f32| Vec3::new(x, y, z);
        self.pane(
            [c(lo.x, lo.y), c(hi.x, lo.y), c(hi.x, hi.y), c(lo.x, hi.y)],
            Vec3::Y,
            rgb,
        );
    }

    /// Moves to a frame standing on the ground at `pos`, turned by `yaw`, keeping the rest of the site.
    pub fn stand(&mut self, pos: Vec2, yaw: f32) {
        let floor = self.terrain.height(pos.x, pos.y);
        self.begin(Site {
            origin: pos,
            yaw,
            floor,
            ..self.site
        });
    }

    /// Converts a world position and height to chunk space.
    pub fn world_pt(&self, p: Vec2, y: f32) -> Vec3 {
        Vec3::new(p.x - self.center.x, y, p.y - self.center.y)
    }

    /// Adds a quad visible from both sides: `top` faces `out`, `under` the opposite way.
    pub fn sheet_quad(&mut self, p: [Vec3; 4], out: Vec3, colors: [[f32; 4]; 2]) {
        #[cfg(test)]
        let start = self.vertex_mark();
        self.quad_out(p, out, colors[0]);
        self.quad_out(p, -out, colors[1]);
        #[cfg(test)]
        self.note(super::audit::Piece::Sheet, start);
    }

    /// Adds a triangle visible from both sides: `colors[0]` faces `out`, `colors[1]` the opposite way.
    pub fn sheet_tri(&mut self, p: [Vec3; 3], out: Vec3, colors: [[f32; 4]; 2]) {
        #[cfg(test)]
        let start = self.vertex_mark();
        self.tri_out(p, out, colors[0]);
        self.tri_out(p, -out, colors[1]);
        #[cfg(test)]
        self.note(super::audit::Piece::Sheet, start);
    }

    /// Adds several boxes of one material, each given as a pair of local corners.
    pub fn boxes(&mut self, list: &[([f32; 3], [f32; 3])], mat: Mat) {
        for (lo, hi) in list {
            self.cuboid(Vec3::from_array(*lo), Vec3::from_array(*hi), mat);
        }
    }
}
