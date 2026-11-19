//! Painting the pictures: an image for each body asked for, a worker for the cells explored
//! since the last one, and the colours it hands back written into the image.

use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, block_on, futures_lite::future::poll_once};

use super::{BYTES, Painted, Raster, Rasters, blank, colour};
use crate::map::explored::Explored;
use crate::map::explored::grid::{COLS, ROWS, centre};
use crate::universe::Universe;
use crate::world::Globe;

/// Most cells one worker paints: about a tenth of a second of ground samples.
const BATCH: usize = 4096;

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

/// Starts a worker for each idle picture whose body has cells explored and not painted.
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
        if raster.job.is_some() || raster.seen == version {
            continue;
        }
        let cells = fresh(raster, explored.words(*body).unwrap_or_default());
        if cells.len() < BATCH {
            raster.seen = version;
        }
        if cells.is_empty() {
            continue;
        }
        let source = raster.globe.clone().ok_or_else(|| universe.planet(*body));
        raster.job = Some(AsyncComputeTaskPool::get().spawn(async move {
            let globe = source.unwrap_or_else(|spec| Arc::new(Globe::of(&spec)));
            let cells = cells
                .into_iter()
                .map(|cell| (cell, colour(&globe, centre(cell))));
            Painted {
                cells: cells.collect(),
                globe,
            }
        }));
    }
}

/// Returns up to [`BATCH`] cells set in `words` and not yet painted, and marks them painted.
fn fresh(raster: &mut Raster, words: &[u64]) -> Vec<u32> {
    let mut cells = Vec::new();
    for (index, (painted, explored)) in raster.painted.iter_mut().zip(words).enumerate() {
        let mut new = explored & !*painted;
        while new != 0 && cells.len() < BATCH {
            let bit = new.trailing_zeros();
            cells.push((index * 64) as u32 + bit);
            *painted |= 1 << bit;
            new &= new - 1;
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
