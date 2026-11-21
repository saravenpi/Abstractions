//! The chamfered block: a box whose edges and corners are cut at forty-five degrees, so every
//! hard part of a body has a bright edge that catches the light.

use bevy::prelude::*;

use super::Rgb;
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
    /// How far the chamfer cuts into each face, in meters. Zero leaves a plain box.
    pub cut: f32,
}

/// The two signs of an axis.
const SIGNS: [f32; 2] = [-1.0, 1.0];

/// Returns the three axes starting at `axis`, in turn.
fn axes(axis: usize) -> [usize; 3] {
    [axis, (axis + 1) % 3, (axis + 2) % 3]
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

    /// Returns the corner at the signs `a`, `b` and `c` along the axes `order`, sitting on the
    /// surface along the axis `on`.
    fn at(&self, order: [usize; 3], signs: [f32; 3], on: usize) -> Vec3 {
        let (mut s, mut outer) = ([0.0; 3], [false; 3]);
        for (axis, sign) in order.into_iter().zip(signs) {
            s[axis] = sign;
        }
        outer[on] = true;
        self.corner(s, outer)
    }

    /// Returns the outward direction along the axes of `order` weighted by `signs`.
    fn out(&self, order: [usize; 3], signs: [f32; 3]) -> Vec3 {
        let mut n = [0.0; 3];
        for (axis, sign) in order.into_iter().zip(signs) {
            n[axis] = sign;
        }
        self.turn * Vec3::from_array(n)
    }
}

/// Adds the box: six faces in `face`, then twelve chamfers and eight corners in `edge` when it
/// has a chamfer.
pub fn block(soup: &mut Soup, b: &Block, face: Rgb, edge: Rgb) {
    for (order, s) in (0..3).flat_map(|axis| SIGNS.map(|s| (axes(axis), s))) {
        let quad = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(a, c)| b.at(order, [s, a, c], order[0]));
        soup.quad(quad, b.out(order, [s, 0.0, 0.0]), face);
    }
    if b.cut <= 0.0 {
        return;
    }
    let pairs = SIGNS.into_iter().flat_map(|a| SIGNS.map(|c| (a, c)));
    for (order, (a, c)) in (0..3).flat_map(|axis| pairs.clone().map(move |p| (axes(axis), p))) {
        let end = |e: f32, on: usize| b.at(order, [e, a, c], on);
        let quad = [
            end(-1.0, order[1]),
            end(1.0, order[1]),
            end(1.0, order[2]),
            end(-1.0, order[2]),
        ];
        soup.quad(quad, b.out(order, [0.0, a, c]), edge);
    }
    for (x, (y, z)) in SIGNS.into_iter().flat_map(|x| pairs.clone().map(move |p| (x, p))) {
        let tri = [0, 1, 2].map(|on| b.at([0, 1, 2], [x, y, z], on));
        soup.tri(tri, b.out([0, 1, 2], [x, y, z]), edge);
    }
}
