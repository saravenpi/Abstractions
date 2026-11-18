//! Region grids: the ground of one atlas cell sampled once on a 17 by 17 grid, before any flow is carved.
//!
//! The climate reads its height, its inland measure and its slope here, and the flows are traced on it,
//! so neither costs a full ground sample.

use super::cell::{Cell, STEPS};
use crate::world::globe::Globe;

/// Nodes along each side of the grid.
const NODES: usize = STEPS + 1;

/// The ground of one cell at the nodes of its grid, row by row along the first chart axis.
pub struct Region {
    /// Ground height above sea level, before flows and ponds.
    height: Vec<f32>,
    /// Continent measure, growing inland.
    inland: Vec<f32>,
}

/// The ground of a region read between its nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Ground height above sea level, before flows and ponds.
    pub height: f32,
    /// Continent measure, growing inland.
    pub inland: f32,
    /// Rise of the height per grid step along the two chart axes.
    pub rise: [f32; 2],
}

impl Region {
    /// Samples the grid of `cell` on `globe`.
    pub fn build(globe: &Globe, cell: Cell) -> Self {
        let n = globe.cells();
        let nodes = (0..NODES * NODES).map(|k| {
            let local = [(k / NODES) as f64, (k % NODES) as f64];
            globe.relief(cell.dir(local, n) * globe.radius())
        });
        let (height, inland) = nodes.unzip();
        Self { height, inland }
    }

    /// Returns the grid read at `local` grid steps into the cell, blended between its four nearest nodes.
    pub fn sample(&self, local: [f64; 2]) -> Sample {
        let top = (STEPS - 1) as f64;
        let [i, j] = local.map(|c| c.floor().clamp(0.0, top));
        let [u, v] = [(local[0] - i) as f32, (local[1] - j) as f32];
        let k = i as usize * NODES + j as usize;
        let corners = |grid: &[f32]| [grid[k], grid[k + NODES], grid[k + 1], grid[k + NODES + 1]];
        let blend = |[a, b, c, d]: [f32; 4]| {
            let (low, high) = (a + (b - a) * u, c + (d - c) * u);
            low + (high - low) * v
        };
        let h = corners(&self.height);
        let rise = [
            (h[1] - h[0]) + (h[3] - h[2] - h[1] + h[0]) * v,
            (h[2] - h[0]) + (h[3] - h[1] - h[2] + h[0]) * u,
        ];
        Sample {
            height: blend(h),
            inland: blend(corners(&self.inland)),
            rise,
        }
    }
}
