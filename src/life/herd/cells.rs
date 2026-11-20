//! The herd grid: each face of the cube sphere cut into equal-angle cells of about 256 metres.
//!
//! A unit direction falls on the face of [`Face::locate`], whose coordinates `u` and `v` run from -1
//! to 1 with equal steps of angle. Each face is cut into `n` by `n` cells, with `n` the face's arc at
//! its centre line, a quarter of the great circle, over 256 metres (1024 for the hunter layer). A cell is named by
//! `(face index in Face::ALL, column, row)`. Cells near face corners are up to a third smaller than
//! at face centres, which the census absorbs as noise.

use std::f64::consts::{FRAC_PI_2, PI};

use bevy::math::{DVec2, DVec3, IVec3};

use super::sphere::east;
use crate::world::globe::{Face, Tangent};

/// Wanted side of a cell in metres.
const SIDE: f64 = 256.0;

/// The cells of one planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Grid {
    /// Cells along each side of a face.
    n: u32,
    /// Sea level radius of the planet in metres.
    pub radius: f64,
}

impl Grid {
    /// Returns the grid of a planet of `radius` metres.
    pub fn of(radius: f64) -> Self {
        Self::sized(radius, SIDE)
    }

    /// Returns the grid of a planet of `radius` metres with cells of about `side` metres.
    pub fn sized(radius: f64, side: f64) -> Self {
        let n = (radius * FRAC_PI_2 / side).round().clamp(1.0, 65_536.0) as u32;
        Self { n, radius }
    }

    /// Returns the mean side of a cell in metres.
    pub fn side(&self) -> f64 {
        self.radius * FRAC_PI_2 / f64::from(self.n)
    }

    /// Returns the mean area of a cell in square kilometres.
    pub fn area(&self) -> f32 {
        (4.0 * PI * self.radius * self.radius / (6.0 * f64::from(self.n).powi(2)) * 1e-6) as f32
    }

    /// Returns the cell that holds unit direction `dir`.
    pub fn cell(&self, dir: DVec3) -> IVec3 {
        let (face, u, v) = Face::locate(dir);
        let index = Face::ALL.iter().position(|f| *f == face).unwrap_or(0);
        let n = f64::from(self.n);
        let quantise = |x: f64| ((x + 1.0) * 0.5 * n).floor().clamp(0.0, n - 1.0) as i32;
        IVec3::new(index as i32, quantise(u), quantise(v))
    }

    /// Returns the unit direction at fractions `(a, b)`, each 0 to 1, across `cell`; `(0.5, 0.5)` is its centre.
    pub fn spot(&self, cell: IVec3, a: f64, b: f64) -> DVec3 {
        let face = Face::ALL[cell.x.clamp(0, 5) as usize];
        let n = f64::from(self.n);
        face.dir(
            (f64::from(cell.y) + a) / n * 2.0 - 1.0,
            (f64::from(cell.z) + b) / n * 2.0 - 1.0,
        )
    }

    /// Returns every cell that has ground within `reach` metres of unit direction `at`, in a stable order.
    pub fn around(&self, at: DVec3, reach: f64) -> Vec<IVec3> {
        let reach = reach.min(self.radius * PI);
        let step = self.side() * 0.5;
        let steps = (reach / step).ceil() as i32;
        let chart = Tangent::new(at, east(at), self.radius, DVec2::ZERO);
        let mut cells: Vec<IVec3> = (-steps..=steps)
            .flat_map(|i| {
                (-steps..=steps).map(move |j| DVec2::new(f64::from(i), f64::from(j)) * step)
            })
            .filter(|p| p.length() <= reach + step)
            .map(|p| self.cell(chart.dir(p)))
            .collect();
        cells.sort_unstable_by_key(|c| c.to_array());
        cells.dedup();
        cells
    }
}
