//! The drainage grid's geometry: cells over the six cube faces, equal angle in both chart axes, and the
//! eight cells around each, across face edges and cube corners.

use bevy::math::DVec3;

use crate::world::globe::Face;

/// Chart steps to the eight neighbours of a cell.
const AROUND: [(f64, f64); 8] = [
    (-1.0, -1.0),
    (-1.0, 0.0),
    (-1.0, 1.0),
    (0.0, -1.0),
    (0.0, 1.0),
    (1.0, -1.0),
    (1.0, 0.0),
    (1.0, 1.0),
];

/// The cells of a drainage grid of `n` cells per face side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    /// Cells along one side of a face.
    pub n: i32,
}

impl Grid {
    /// Returns how many cells the six faces hold.
    pub fn len(self) -> usize {
        6 * (self.n * self.n) as usize
    }

    /// Returns the unit direction of the middle of cell `index`.
    pub fn dir(self, index: usize) -> DVec3 {
        let (face, i, j) = self.split(index);
        self.cell_dir(face, f64::from(i), f64::from(j))
    }

    /// Returns the cell holding unit direction `dir`.
    pub fn find(self, dir: DVec3) -> usize {
        let (face, u, v) = Face::locate(dir);
        let at =
            |c: f64| (((c + 1.0) * 0.5 * f64::from(self.n)).floor() as i32).clamp(0, self.n - 1);
        self.join(face, at(u), at(v))
    }

    /// Returns the cells around cell `index`: its eight neighbours, some repeated at a cube corner.
    pub fn around(self, index: usize) -> [usize; 8] {
        let (face, i, j) = self.split(index);
        AROUND.map(|(di, dj)| {
            let dir = self.cell_dir(face, f64::from(i) + di, f64::from(j) + dj);
            self.find(dir)
        })
    }

    /// Returns the face and the column and row of cell `index`.
    fn split(self, index: usize) -> (Face, i32, i32) {
        let side = (self.n * self.n) as usize;
        let rest = (index % side) as i32;
        (Face::ALL[index / side], rest / self.n, rest % self.n)
    }

    /// Returns the cell of `face` at column `i` and row `j`.
    fn join(self, face: Face, i: i32, j: i32) -> usize {
        let at = Face::ALL.iter().position(|f| *f == face).unwrap_or(0);
        at * (self.n * self.n) as usize + (i * self.n + j) as usize
    }

    /// Returns the unit direction of the middle of column `i` and row `j` of `face`, also past its edges.
    fn cell_dir(self, face: Face, i: f64, j: f64) -> DVec3 {
        let at = |c: f64| -1.0 + (c + 0.5) * 2.0 / f64::from(self.n);
        face.dir(at(i), at(j))
    }
}
