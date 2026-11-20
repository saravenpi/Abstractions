//! The chamfered block: a box whose edges and corners are cut at forty-five degrees, so every
//! hard part of a body has a bright edge that catches the light.

use bevy::prelude::*;

use super::soup::Soup;

/// A chamfered box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Block {
    /// Middle of the box.
    pub at: Vec3,
    /// Half extents before `turn`.
    pub half: Vec3,
    /// Turn of the box about its middle.
    pub turn: Quat,
    /// How far the chamfer cuts into each face, in meters.
    pub cut: f32,
}

impl Block {
    /// Returns a box of half extents `half` at `at`, not turned, with a chamfer `cut`.
    pub fn new(at: Vec3, half: Vec3, cut: f32) -> Self {
        Self {
            at,
            half,
            turn: Quat::IDENTITY,
            cut: cut.min(half.min_element() * 0.9),
        }
    }

    /// Returns the box turned by `turn`.
    pub fn turned(mut self, turn: Quat) -> Self {
        self.turn = turn;
        self
    }

    /// Returns the corner of the signs `s` whose axes marked in `outer` sit on the box surface and
    /// whose other axes are pulled in by the chamfer.
    fn corner(&self, s: [f32; 3], outer: [bool; 3]) -> Vec3 {
        let at = |i: usize| s[i] * (self.half[i] - if outer[i] { 0.0 } else { self.cut });
        self.at + self.turn * Vec3::new(at(0), at(1), at(2))
    }
}

/// The two signs of an axis.
const SIGNS: [f32; 2] = [-1.0, 1.0];

/// Adds the box: six faces in `face`, then twelve bevels and eight corners in `edge`.
pub fn block(soup: &mut Soup, b: &Block, face: [f32; 3], edge: [f32; 3]) {
    for axis in 0..3 {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        for s in SIGNS {
            let quad = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, c)| {
                let mut sg = [0.0; 3];
                (sg[axis], sg[u], sg[v]) = (s, a, c);
                let mut outer = [false; 3];
                outer[axis] = true;
                b.corner(sg, outer)
            });
            let mut n = [0.0; 3];
            n[axis] = s;
            soup.quad(quad, b.turn * Vec3::from_array(n), face);
        }
    }
    bevels(soup, b, edge);
}

/// Adds the twelve bevel strips and the eight corner triangles of the box.
fn bevels(soup: &mut Soup, b: &Block, edge: [f32; 3]) {
    for axis in 0..3 {
        let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
        for (a, c) in SIGNS.into_iter().flat_map(|a| SIGNS.map(|c| (a, c))) {
            let end = |e: f32| {
                let mut sg = [0.0; 3];
                (sg[axis], sg[u], sg[v]) = (e, a, c);
                let (mut on_u, mut on_v) = ([false; 3], [false; 3]);
                (on_u[u], on_v[v]) = (true, true);
                (b.corner(sg, on_u), b.corner(sg, on_v))
            };
            let (p0, q0) = end(-1.0);
            let (p1, q1) = end(1.0);
            let mut n = [0.0; 3];
            (n[u], n[v]) = (a, c);
            soup.quad([p0, p1, q1, q0], b.turn * Vec3::from_array(n), edge);
        }
    }
    for (x, y, z) in SIGNS.into_iter().flat_map(|x| SIGNS.into_iter().flat_map(move |y| SIGNS.map(move |z| (x, y, z)))) {
        let s = [x, y, z];
        let tri = [0, 1, 2].map(|i| {
            let mut outer = [false; 3];
            outer[i] = true;
            b.corner(s, outer)
        });
        soup.tri(tri, b.turn * Vec3::from_array(s), edge);
    }
}
