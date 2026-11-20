//! A per-thread memory of the site found in each cell, so that neighboring chunks and per-frame queries do
//! not search the same cell twice.

use std::cell::RefCell;
use std::collections::HashMap;

use bevy::prelude::*;

use super::site::{CELL, Site};
use super::village::SPAN;
use crate::world::globe::Chart;
use crate::world::{Globe, Terrain};

/// Number of remembered cells after which the memory is emptied.
const LIMIT: usize = 16_384;

/// Which grid a remembered cell belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Grid {
    /// The cells that hold at most one structure.
    Structures,
    /// The coarse cells that hold at most one village.
    Villages,
}

impl Grid {
    /// Returns the side of one cell of the grid in chart units.
    pub fn side(self) -> f32 {
        match self {
            Grid::Structures => CELL,
            Grid::Villages => SPAN as f32 * CELL,
        }
    }
}

/// What tells one cell of one chart of one planet from every other. The cell is counted in whole cells from
/// the middle of the chart, never by a chart position, so every view of a face shares one entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    seed: u64,
    radius: u64,
    face: u8,
    scale: u64,
    reserved: bool,
    class: u8,
    ocean: u64,
    air: [u64; 4],
    grid: Grid,
    cell: IVec2,
}

thread_local! {
    static CACHE: RefCell<HashMap<Key, Option<Site>>> = RefCell::new(HashMap::new());
}

/// Returns the bits of what shapes the climate of `globe` besides its seed, size and sea, which the
/// biome and the village rules read.
fn air(globe: &Globe) -> [u64; 4] {
    let record = globe.record();
    [
        record.temperature,
        record.pressure,
        record.tilt,
        globe.gravity(),
    ]
    .map(f64::to_bits)
}

/// Returns the key of `cell`, counted from the middle of the chart of `terrain`, or nothing on a chart whose
/// cells are not remembered.
fn key(terrain: &Terrain, grid: Grid, cell: IVec2) -> Option<Key> {
    let (face, scale) = match terrain.chart() {
        Chart::Flat => (u8::MAX, 0),
        Chart::Face { face, scale, .. } => (face as u8, scale.to_bits()),
        Chart::Tangent(_) => return None,
    };
    let globe = terrain.globe();
    Some(Key {
        seed: terrain.seed(),
        radius: globe.radius().to_bits(),
        face,
        scale,
        reserved: globe.reserve().is_some(),
        class: globe.record().class as u8,
        ocean: globe.record().ocean.to_bits(),
        air: air(globe),
        grid,
        cell,
    })
}

/// Returns what `find` returns for `cell` of `grid` on `terrain`, searching only the first time on each thread.
///
/// `find` gets the home view of the cell and the cell in it, not `terrain`: the site found is the same from
/// every view of the face. It comes back at its chart position in `terrain`.
pub fn memo(
    terrain: &Terrain,
    at: (Grid, IVec2),
    find: impl FnOnce(&Terrain, IVec2) -> Option<Site>,
) -> Option<Site> {
    let home = terrain.home(at.1, at.0.side());
    let back = |site: Site| Site {
        origin: site.origin + home.offset,
        ..site
    };
    let Some(key) = key(&home.view, at.0, home.id) else {
        return find(terrain, at.1);
    };
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&key).copied()) {
        return hit.map(back);
    }
    let found = find(&home.view, home.cell);
    CACHE.with(|c| {
        let mut map = c.borrow_mut();
        if map.len() >= LIMIT {
            map.clear();
        }
        map.insert(key, found);
    });
    found.map(back)
}
