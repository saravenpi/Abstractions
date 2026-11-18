//! Atlas cells: each cube face cut into equal squares of its gnomonic chart, at most 2 km wide.
//!
//! The gnomonic chart, the ratio of a direction's components, needs no trigonometry, so finding the cell
//! of a ground sample costs a few multiplications. Its squares are widest at the middle of a face and
//! half as wide at the corners, which the count of cells is sized for.

use bevy::math::DVec3;

use crate::world::globe::Face;

/// Widest a cell may be, in metres along the ground.
const WIDTH: f64 = 2000.0;

/// Grid steps along each side of a cell: its height grid has one more node per side.
pub const STEPS: usize = 16;

/// One cell of the atlas: square `(i, j)` of `face`, counted from the corner at chart `(-1, -1)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    /// The cube face the cell lies on.
    pub face: Face,
    /// Column along the face chart's first axis.
    pub i: i32,
    /// Row along the face chart's second axis.
    pub j: i32,
}

/// Returns the face whose axis is nearest unit direction `dir`.
fn face_of(dir: DVec3) -> Face {
    let a = dir.abs();
    if a.x >= a.y && a.x >= a.z {
        [Face::NegX, Face::PosX][usize::from(dir.x > 0.0)]
    } else if a.y >= a.z {
        [Face::NegY, Face::PosY][usize::from(dir.y > 0.0)]
    } else {
        [Face::NegZ, Face::PosZ][usize::from(dir.z > 0.0)]
    }
}

impl Cell {
    /// Returns the cells along one side of a face on a planet of `radius` metres: the middle of a face,
    /// where the chart is widest, has squares no wider than [`WIDTH`].
    pub fn count(radius: f64) -> i32 {
        (2.0 * radius / WIDTH).ceil().clamp(1.0, 1e6) as i32
    }

    /// Returns the cell holding unit direction `dir` on a planet of `n` cells per face side, and where `dir`
    /// lies in it in grid steps, each from 0 to [`STEPS`].
    pub fn locate(dir: DVec3, n: i32) -> (Cell, [f64; 2]) {
        let face = face_of(dir);
        let [normal, a, b] = face.axes();
        let depth = dir.dot(normal);
        Self::on(face, [dir.dot(a) / depth, dir.dot(b) / depth], n)
    }

    /// Returns the cell of `face` holding gnomonic chart point `t` and where the point lies in it in grid
    /// steps, the point being clamped to the face.
    pub fn on(face: Face, t: [f64; 2], n: i32) -> (Cell, [f64; 2]) {
        let at = t.map(|c| (c + 1.0) * 0.5 * f64::from(n));
        let index = at.map(|c| (c.floor() as i32).clamp(0, n - 1));
        let local =
            [0, 1].map(|k| ((at[k] - f64::from(index[k])) * STEPS as f64).clamp(0.0, STEPS as f64));
        let cell = Cell {
            face,
            i: index[0],
            j: index[1],
        };
        (cell, local)
    }

    /// Returns the gnomonic chart point of the place `local` grid steps into this cell, also past its sides.
    pub fn chart(self, local: [f64; 2], n: i32) -> [f64; 2] {
        let size = 2.0 / f64::from(n);
        let corner = [self.i, self.j].map(|k| -1.0 + size * f64::from(k));
        [0, 1].map(|k| corner[k] + size * local[k] / STEPS as f64)
    }

    /// Returns the unit direction of gnomonic chart point `t` of the face of this cell.
    pub fn at(self, t: [f64; 2]) -> DVec3 {
        let [normal, a, b] = self.face.axes();
        (normal + a * t[0] + b * t[1]).normalize()
    }

    /// Returns the unit direction of the place `local` grid steps into this cell, also past its sides.
    pub fn dir(self, local: [f64; 2], n: i32) -> DVec3 {
        self.at(self.chart(local, n))
    }

    /// Returns the widths in metres of this cell across its middle along both chart axes, on a planet of
    /// `n` cells per face side and `radius` metres.
    pub fn size(self, n: i32, radius: f64) -> [f64; 2] {
        let (low, mid, high) = (0.0, STEPS as f64 * 0.5, STEPS as f64);
        let across = |a: [f64; 2], b: [f64; 2]| (self.dir(a, n) - self.dir(b, n)).length() * radius;
        [
            across([high, mid], [low, mid]),
            across([mid, high], [mid, low]),
        ]
    }
}
