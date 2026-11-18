//! The climate over one chunk: read once per cell of a coarse grid, then looked up by every try.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::relief::Relief;
use crate::climate::Climate;
use crate::world::lands::thrive::climate_on;
use crate::world::{CHUNK_SIZE, Terrain};

#[cfg(test)]
thread_local! {
    /// True while a test holds the climate still on this thread.
    static CALM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Cells along one side of the chunk at which the climate is read.
const CELLS: usize = 4;

/// The climate of the cells of one chunk on a globe.
pub struct Air {
    /// Climate at the centre of each cell, the first index along x and the second along z.
    cells: [[Climate; CELLS]; CELLS],
    /// Chart position of the low corner of the chunk.
    corner: Vec2,
}

/// Returns the rise per metre, in planet axes, of ground whose gradient is `grade` metres per chart
/// unit along the chart axes at `w` of `terrain`.
fn rise(terrain: &Terrain, w: Vec2, grade: Vec2) -> DVec3 {
    let (chart, radius) = (terrain.chart(), terrain.globe().radius());
    let here = chart.dir(w.x, w.y);
    let axis = |by: Vec2| (chart.dir(w.x + by.x, w.y + by.y) - here) * radius;
    let (x, z) = (axis(Vec2::X), axis(Vec2::Y));
    x * f64::from(grade.x) / x.length_squared() + z * f64::from(grade.y) / z.length_squared()
}

impl Air {
    /// Reads the climate over chunk `coord` of `terrain`, standing on its visible `relief`.
    pub fn new(terrain: &Terrain, coord: IVec2, relief: &Relief) -> Self {
        let corner = coord.as_vec2() * CHUNK_SIZE;
        let step = CHUNK_SIZE / CELLS as f32;
        let read = |i: usize, j: usize| {
            let w = corner + (Vec2::new(i as f32, j as f32) + 0.5) * step;
            let (height, grade) = relief.at(w);
            let dir = terrain.chart().dir(w.x, w.y);
            climate_on(terrain.globe(), dir, height, rise(terrain, w, grade))
        };
        Self {
            cells: std::array::from_fn(|i| std::array::from_fn(|j| read(i, j))),
            corner,
        }
    }

    /// Returns the climate of the cell that holds chart position `w` of the chunk.
    pub fn at(&self, w: Vec2) -> &Climate {
        let cell = ((w - self.corner) / (CHUNK_SIZE / CELLS as f32)).floor();
        let [i, j] = cell
            .clamp(Vec2::ZERO, Vec2::splat((CELLS - 1) as f32))
            .to_array();
        &self.cells[i as usize][j as usize]
    }
}

/// Returns what `work` returns when scattered with the climate held still on this thread, every
/// chunk growing as if it were the heart of its biome. Tests that measure the chart use it.
#[cfg(test)]
pub fn calm<T>(work: impl FnOnce() -> T) -> T {
    CALM.with(|calm| calm.set(true));
    let done = work();
    CALM.with(|calm| calm.set(false));
    done
}

/// Returns true while a test holds the climate still on this thread.
#[cfg(test)]
pub fn still() -> bool {
    CALM.with(std::cell::Cell::get)
}

/// Returns false: the game never holds the climate still.
#[cfg(not(test))]
pub fn still() -> bool {
    false
}
