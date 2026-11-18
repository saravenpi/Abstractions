//! Pit maps: for one atlas cell, every crater that reaches into it, sorted into 32 by 32 bins.
//!
//! Craters are far smaller than a cell, so the craters of the cell and of the cells around it are enough.

use bevy::math::DVec3;

use super::cell::{Cell, STEPS};
use crate::world::globe::Globe;
use crate::world::globe::craters::{Crater, FOOT, craters};

/// Bins along each side of a cell.
const BINS: usize = 32;

/// Half cells gathered around the middle of a cell on each side.
const AROUND: i32 = 3;

/// The craters reaching into one cell, binned.
pub struct PitMap {
    /// Every crater near the cell.
    craters: Vec<Crater>,
    /// Where the list of each bin starts in `items`, and one more entry for the end.
    starts: Vec<u32>,
    /// The craters of every bin, one bin after the other.
    items: Vec<u32>,
}

/// Returns where unit direction `dir` lies in `cell`, in grid steps, also past its sides, or nothing when
/// it lies on the far side of the globe from the cell's face.
pub fn local_of(cell: Cell, dir: DVec3, n: i32) -> Option<[f64; 2]> {
    let [normal, a, b] = cell.face.axes();
    let depth = dir.dot(normal);
    let corner = cell.chart([0.0, 0.0], n);
    let size = 2.0 / f64::from(n);
    let t = [dir.dot(a) / depth, dir.dot(b) / depth];
    (depth > 0.0).then(|| [0, 1].map(|k| (t[k] - corner[k]) / size * STEPS as f64))
}

/// Returns the cells within `reach` half cells of the middle of `cell` on a planet of `n` cells per face
/// side, across face edges too.
pub fn around(cell: Cell, n: i32, reach: i32) -> Vec<Cell> {
    let mut cells: Vec<Cell> = Vec::new();
    for (a, b) in (-reach..=reach).flat_map(|a| (-reach..=reach).map(move |b| (a, b))) {
        let local = [a, b].map(|c| (0.5 + f64::from(c) * 0.5) * STEPS as f64);
        let t = cell.chart(local, n).map(|c| c.clamp(-3.0, 3.0));
        let (holder, _) = Cell::locate(cell.at(t), n);
        if !cells.contains(&holder) {
            cells.push(holder);
        }
    }
    cells
}

impl PitMap {
    /// Builds the map of `cell` from the craters of the cells around it.
    pub fn build(globe: &Globe, cell: Cell) -> Self {
        let n = globe.cells();
        let craters: Vec<Crater> = around(cell, n, AROUND)
            .iter()
            .flat_map(|&c| craters(globe, c))
            .collect();
        let [wide, tall] = cell.size(n, globe.radius());
        let per_bin = wide.min(tall) / BINS as f64;
        let mut bins = vec![Vec::new(); BINS * BINS];
        for (index, crater) in craters.iter().enumerate() {
            let Some(local) = local_of(cell, crater.centre.normalize(), n) else {
                continue;
            };
            let reach = (f64::from(FOOT * 0.5 * crater.size) / per_bin).ceil() as i64 + 1;
            let at = local.map(|c| (c * (BINS / STEPS) as f64).floor() as i64);
            let span = |k: usize| (at[k] - reach).max(0)..=(at[k] + reach).min(BINS as i64 - 1);
            for (i, j) in span(0).flat_map(|i| span(1).map(move |j| (i, j))) {
                bins[i as usize * BINS + j as usize].push(index as u32);
            }
        }
        let mut starts = vec![0];
        bins.iter()
            .for_each(|b| starts.push(starts[starts.len() - 1] + b.len() as u32));
        Self {
            craters,
            starts,
            items: bins.concat(),
        }
    }

    /// Returns the height the craters add at planet point `p`, `local` grid steps into the cell.
    pub fn depth(&self, p: DVec3, local: [f64; 2]) -> f32 {
        if self.items.is_empty() {
            return 0.0;
        }
        let bin = local.map(|c| ((c * (BINS / STEPS) as f64) as usize).min(BINS - 1));
        let k = bin[0] * BINS + bin[1];
        let range = self.starts[k] as usize..self.starts[k + 1] as usize;
        self.items[range]
            .iter()
            .map(|&c| self.craters[c as usize].depth(p))
            .sum()
    }
}
