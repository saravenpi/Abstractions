//! The visible ground of one chunk: the heights of its finest terrain mesh, read without sampling the planet again.

use bevy::prelude::*;

use crate::world::CHUNK_SIZE;

/// The terrain mesh of one chunk as a height field over its chart square.
pub struct Relief<'a> {
    /// Heights of the grid vertices, the first index along x and the second along z.
    heights: &'a [Vec<f32>],
    /// Chart position of vertex `(0, 0)`.
    corner: Vec2,
    /// Number of cells along one side.
    cells: usize,
}

impl<'a> Relief<'a> {
    /// Returns the relief of chunk `coord` over the `heights` of its mesh vertices.
    pub fn new(coord: IVec2, heights: &'a [Vec<f32>]) -> Self {
        Self {
            heights,
            corner: coord.as_vec2() * CHUNK_SIZE,
            cells: heights.len() - 1,
        }
    }

    /// Returns true when chart position `w` lies in the square of the chunk, its low edges included and its high
    /// edges left to the next chunk, so that every point of a face belongs to exactly one chunk.
    pub fn holds(&self, w: Vec2) -> bool {
        let d = w - self.corner;
        d.min_element() >= 0.0 && d.max_element() < CHUNK_SIZE
    }

    /// Returns true when one of 25 vertices spread over the chunk lies above `floor`.
    pub fn shallow(&self, floor: f32) -> bool {
        let at = |k: usize| (k * self.cells) / 4;
        (0..25).any(|i| self.heights[at(i % 5)][at(i / 5)] > floor)
    }

    /// Returns the height of the mesh at chart position `w` of the chunk, and the gradient of the triangle
    /// there, in metres per chart unit along x and z.
    pub fn at(&self, w: Vec2) -> (f32, Vec2) {
        let step = CHUNK_SIZE / self.cells as f32;
        let last = (self.cells - 1) as f32;
        let cell = ((w - self.corner) / step)
            .floor()
            .clamp(Vec2::ZERO, Vec2::splat(last));
        let part = (w - self.corner) / step - cell;
        let (i, j) = (cell.x as usize, cell.y as usize);
        let h = |a: usize, b: usize| self.heights[i + a][j + b];
        if part.x + part.y <= 1.0 {
            let rise = Vec2::new(h(1, 0) - h(0, 0), h(0, 1) - h(0, 0));
            (h(0, 0) + part.dot(rise), rise / step)
        } else {
            let rise = Vec2::new(h(1, 1) - h(0, 1), h(1, 1) - h(1, 0));
            (h(1, 1) - (Vec2::ONE - part).dot(rise), rise / step)
        }
    }
}
