//! Sampling a span: the direction, height and planet position of each of its grid vertices.

use bevy::math::DVec3;

use super::lattice;
use super::span::Span;

impl Span {
    /// Returns the unit direction away from the planet centre at vertex `(i, j)`.
    pub fn dir(&self, i: usize, j: usize) -> DVec3 {
        match self.node {
            Some(id) => lattice::dir(id, i, j, self.n),
            None => {
                let at = self.chart(i, j, 0.0);
                self.terrain.up(at.x, at.y)
            }
        }
    }

    /// Returns the ground height above sea level at vertex `(i, j)`.
    pub fn height(&self, i: usize, j: usize) -> f32 {
        match self.node {
            Some(_) => self.terrain.globe().height(self.dir(i, j)),
            None => {
                let at = self.chart(i, j, 0.0);
                self.terrain.height(at.x, at.y)
            }
        }
    }

    /// Returns the planet position of vertex `(i, j)` raised `y` metres above sea level.
    pub fn point(&self, i: usize, j: usize, y: f32) -> DVec3 {
        match self.node {
            Some(_) => self.dir(i, j) * (self.terrain.globe().radius() + f64::from(y)),
            None => {
                let at = self.chart(i, j, 0.0);
                self.terrain.embed(at.x, y, at.y)
            }
        }
    }

    /// Samples the ground height at every vertex; the first index runs along x, the second along z.
    pub fn heights(&self) -> Vec<Vec<f32>> {
        (0..=self.n)
            .map(|i| (0..=self.n).map(|j| self.height(i, j)).collect())
            .collect()
    }
}
