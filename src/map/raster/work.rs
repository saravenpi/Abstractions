//! Painting the pictures: an image for each body asked for, a worker for the cells explored
//! since the last one, and the colours it hands back written into the image.

use std::sync::Arc;

use abstractions_net::leaves::Leaves;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, futures_lite::future::poll_once};

use super::grid::{COLS, ROWS, cell_of, centre};
use super::{BYTES, Painted, Rasters, blank, colour};
use crate::map::explored::Explored;
use crate::map::explored::grid::centre as leaf_centre;
use crate::universe::Universe;
use crate::world::Globe;

/// Gives every body asked for its blank image.
pub(super) fn open(mut rasters: ResMut<Rasters>, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else {
        return;
    };
    for raster in rasters.bodies.values_mut() {
        if raster.image.is_none() {
            raster.image = Some(images.add(blank()));
        }
    }
}

/// Starts a worker for each idle picture whose body has leaves explored and not yet looked at.
pub(super) fn plan(
    mut rasters: ResMut<Rasters>,
    explored: Res<Explored>,
    universe: Option<Res<Universe>>,
) {
    let Some(universe) = universe.as_deref() else {
        return;
    };
    for (body, raster) in &mut rasters.bodies {
        let version = explored.version(*body);
        let Some(leaves) = explored.leaves(*body) else {
            continue;
        };
        if raster.job.is_some() || raster.seen == version {
            continue;
        }
        raster.seen = version;
        let fresh = raster.painted.merge(leaves);
        let lit = raster.lit.clone();
        let source = raster.globe.clone().ok_or_else(|| universe.planet(*body));
        raster.job = Some(AsyncComputeTaskPool::get().spawn(async move {
            let globe = source.unwrap_or_else(|spec| Arc::new(Globe::of(&spec)));
            let cells = new_cells(lit, &fresh).into_iter();
            Painted {
                cells: cells
                    .map(|cell| (cell, colour(&globe, centre(cell))))
                    .collect(),
                globe,
            }
        }));
    }
}

/// Returns the cells of the picture that hold a leaf of `fresh` and are not set in `lit`, each
/// once.
fn new_cells(mut lit: Vec<u64>, fresh: &Leaves) -> Vec<u32> {
    let mut cells = Vec::new();
    for leaf in fresh.iter() {
        let cell = cell_of(leaf_centre(leaf)) as usize;
        let (word, bit) = (cell / 64, 1u64 << (cell % 64));
        if lit[word] & bit == 0 {
            lit[word] |= bit;
            cells.push(cell as u32);
        }
    }
    cells
}

/// Writes the cells a worker finished into the picture of its body.
pub(super) fn collect(mut rasters: ResMut<Rasters>, images: Option<ResMut<Assets<Image>>>) {
    let Some(mut images) = images else {
        return;
    };
    for raster in rasters.bodies.values_mut() {
        let Some(done) = raster.job.as_mut().and_then(|job| block_on(poll_once(job))) else {
            continue;
        };
        raster.job = None;
        for (cell, _) in &done.cells {
            raster.lit[*cell as usize / 64] |= 1 << (cell % 64);
        }
        if let Some(mut image) = raster
            .image
            .as_ref()
            .and_then(|image| images.get_mut(image))
        {
            write(&mut image, &done.cells);
        }
        raster.globe = Some(done.globe);
    }
}

/// Writes the pixel of each cell into `image`.
fn write(image: &mut Image, cells: &[(u32, [u8; BYTES])]) {
    let Some(data) = image.data.as_mut() else {
        return;
    };
    for (cell, pixel) in cells {
        let at = offset(*cell);
        data[at..at + BYTES].copy_from_slice(pixel);
    }
}

/// Returns the byte offset in the picture of the pixel of `cell`, the north at the top.
pub(super) fn offset(cell: u32) -> usize {
    let (row, column) = (cell as usize / COLS, cell as usize % COLS);
    ((ROWS - 1 - row) * COLS + column) * BYTES
}
