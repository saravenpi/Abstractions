//! The drainage grid: one coarse height and one flow direction per cell of a grid of 6 by 256 by 256 cells
//! over the cube sphere, about 40 km wide on an Earth.
//!
//! It is a pure function of the planet, built once per planet and shared by every copy of its globe and
//! by every globe of the same planet made later, see [`shared`]. A priority flood from the sea gives every
//! land cell the neighbour its water leaves by, so every path ends in the sea or in a closed basin, and the
//! area draining through a cell tells the trunks of the rivers.
//! The trunks guide the local flows of the atlas, and the shores of the land near mild latitudes are where a
//! new game starts.

mod course;
mod flood;
mod grid;
mod query;
mod shared;
mod shore;
#[cfg(test)]
mod tests_grid;
pub mod tune;

use std::sync::{Arc, OnceLock};

use bevy::math::DVec3;
use bevy::tasks::AsyncComputeTaskPool;

use super::Globe;
use flood::flood;
use grid::Grid;
use tune::SIDE;

/// The drainage of one planet.
pub struct Drainage {
    /// The cells.
    grid: Grid,
    /// The large height of each cell in metres above the sea.
    height: Vec<f32>,
    /// The cell each one drains into, itself at the sea and at the bottom of a basin.
    next: Vec<u32>,
    /// Land cells draining through each cell.
    area: Vec<u32>,
    /// Height of the sea in the same unit: nothing is under it on a dry planet.
    sea: f32,
    /// The places a new game may start at, worked out when first asked for.
    starts: OnceLock<Vec<DVec3>>,
}

/// Returns the large height of `globe` in unit direction `dir`: the plates and the regional band where the
/// planet has them, the shaped ground of a small one otherwise.
fn level(globe: &Globe, dir: DVec3) -> f32 {
    globe
        .large_height(dir)
        .unwrap_or_else(|| globe.shaped(dir * globe.radius()))
}

/// Reads the heights of the cells of `grid` from cell `first` on into `part`.
fn fill(globe: &Globe, grid: Grid, first: usize, part: &mut [f32]) {
    for (k, height) in part.iter_mut().enumerate() {
        *height = level(globe, grid.dir(first + k));
    }
}

/// Returns the heights of every cell of `grid` on `globe`, read on `threads` threads.
fn heights(globe: &Globe, grid: Grid, threads: usize) -> Vec<f32> {
    let mut out = vec![0.0; grid.len()];
    if threads == 1 {
        fill(globe, grid, 0, &mut out);
        return out;
    }
    let size = out.len().div_ceil(threads);
    std::thread::scope(|scope| {
        for (t, part) in out.chunks_mut(size).enumerate() {
            scope.spawn(move || fill(globe, grid, t * size, part));
        }
    });
    out
}

impl Drainage {
    /// Builds the drainage of `globe`: [`SIDE`] cells along a face, or one per atlas cell on a planet too
    /// small for that, reading the heights on as many threads as the machine has.
    pub fn build(globe: &Globe) -> Self {
        Self::build_on(
            globe,
            std::thread::available_parallelism().map_or(1, usize::from),
        )
    }

    /// Builds the drainage of `globe` reading the heights on `threads` threads.
    pub fn build_on(globe: &Globe, threads: usize) -> Self {
        let grid = Grid {
            n: globe.cells().clamp(1, SIDE),
        };
        let height = heights(globe, grid, threads.max(1));
        let sea = globe.sea_level();
        let flood = flood(grid, &height, sea);
        Self {
            grid,
            height,
            next: flood.next,
            area: flood.area,
            sea,
            starts: OnceLock::new(),
        }
    }
}

impl Globe {
    /// Returns the drainage of the planet, built on first use and shared by every copy of this globe. The
    /// first caller waits for the build: ask for it from a task, or call [`Globe::warm_drainage`] ahead.
    pub fn drainage(&self) -> Arc<Drainage> {
        self.atlas
            .drainage
            .get_or_init(|| shared::of(self.atlas.print, || Drainage::build(self)))
            .clone()
    }

    /// Starts building the drainage on the async compute pool when it is not built yet and the pool runs,
    /// so that no frame waits for it.
    pub fn warm_drainage(&self) {
        if !self.has_ground() || self.atlas.drainage.get().is_some() {
            return;
        }
        if let Some(pool) = AsyncComputeTaskPool::try_get() {
            let globe = self.clone();
            pool.spawn(async move {
                globe.drainage();
            })
            .detach();
        }
    }
}
