//! Patches: the grid vertices of one span, placed in the frame its mesh is stored in.

use bevy::prelude::*;

use super::frame::Frame;
use super::span::Span;

/// The vertices of a span in a frame, with the height each one is drawn at.
pub struct Patch {
    n: usize,
    pos: Vec<Vec3>,
    up: Vec<Vec3>,
    height: Vec<f32>,
}

impl Patch {
    /// Places the vertices of `span` in `frame` at `heights`, none drawn lower than `floor`.
    pub fn new(span: &Span, frame: &Frame, heights: &[Vec<f32>], floor: f32) -> Self {
        let n = heights.len() - 1;
        let cells = || (0..=n).flat_map(|i| (0..=n).map(move |j| (i, j)));
        let height: Vec<f32> = cells().map(|(i, j)| heights[i][j].max(floor)).collect();
        let at = |(i, j): (usize, usize)| frame.local(span.point(i, j, height[i * (n + 1) + j]));
        Self {
            n,
            pos: cells().map(at).collect(),
            up: cells().map(|(i, j)| frame.turn(span.dir(i, j))).collect(),
            height,
        }
    }

    /// Returns the number of cells along one side.
    pub fn cells(&self) -> usize {
        self.n
    }

    /// Returns the frame position of vertex `(i, j)`.
    pub fn at(&self, i: usize, j: usize) -> Vec3 {
        self.pos[i * (self.n + 1) + j]
    }

    /// Returns the direction away from the planet centre at vertex `(i, j)`, in frame axes.
    pub fn up(&self, i: usize, j: usize) -> Vec3 {
        self.up[i * (self.n + 1) + j]
    }

    /// Returns the height above sea level vertex `(i, j)` is drawn at.
    pub fn height(&self, i: usize, j: usize) -> f32 {
        self.height[i * (self.n + 1) + j]
    }

    /// Returns the two triangles of every cell as indices into [`Patch::positions`].
    pub fn triangles(&self) -> Vec<[u32; 3]> {
        self.triangles_kept(|_, _, _| true)
    }

    /// Returns the triangles of every cell kept by `keep(i, j, half)` as indices into [`Patch::positions`].
    pub fn triangles_kept(&self, keep: impl Fn(usize, usize, usize) -> bool) -> Vec<[u32; 3]> {
        let row = self.n as u32 + 1;
        let mut tris = Vec::with_capacity(self.n * self.n * 2);
        for i in 0..self.n {
            for j in 0..self.n {
                let (a, b) = (i as u32 * row + j as u32, (i as u32 + 1) * row + j as u32);
                if keep(i, j, 0) {
                    tris.push([a, a + 1, b]);
                }
                if keep(i, j, 1) {
                    tris.push([b, a + 1, b + 1]);
                }
            }
        }
        tris
    }

    /// Returns the frame positions of all vertices, vertex `(i, j)` at index `i * (cells + 1) + j`.
    pub fn positions(&self) -> &[Vec3] {
        &self.pos
    }
}
