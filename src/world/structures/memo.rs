//! A per-thread memory of the site found in each cell, so that neighboring chunks and per-frame queries do
//! not search the same cell twice.

use std::cell::RefCell;
use std::collections::HashMap;

use bevy::prelude::*;

use super::site::Site;
use crate::world::Terrain;
use crate::world::globe::Chart;

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

/// What tells one cell of one chart of one planet from every other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    seed: u64,
    radius: u64,
    face: u8,
    scale: u64,
    reserved: bool,
    grid: Grid,
    cell: IVec2,
}

thread_local! {
    static CACHE: RefCell<HashMap<Key, Option<Site>>> = RefCell::new(HashMap::new());
}

/// Returns the key of `cell` on the chart of `terrain`, or nothing on a chart whose cells are not remembered.
fn key(terrain: &Terrain, grid: Grid, cell: IVec2) -> Option<Key> {
    let (face, scale) = match terrain.chart() {
        Chart::Flat => (u8::MAX, 0),
        Chart::Face { face, scale } => (face as u8, scale.to_bits()),
        Chart::Tangent(_) => return None,
    };
    Some(Key {
        seed: terrain.seed(),
        radius: terrain.globe().radius().to_bits(),
        face,
        scale,
        reserved: terrain.globe().reserve().is_some(),
        grid,
        cell,
    })
}

/// Returns what `find` returns for `cell` of `grid` on `terrain`, searching only the first time on each thread.
pub fn memo(
    terrain: &Terrain,
    at: (Grid, IVec2),
    find: impl FnOnce() -> Option<Site>,
) -> Option<Site> {
    let Some(key) = key(terrain, at.0, at.1) else {
        return find();
    };
    if let Some(hit) = CACHE.with(|c| c.borrow().get(&key).copied()) {
        return hit;
    }
    let found = find();
    CACHE.with(|c| {
        let mut map = c.borrow_mut();
        if map.len() >= LIMIT {
            map.clear();
        }
        map.insert(key, found);
    });
    found
}
