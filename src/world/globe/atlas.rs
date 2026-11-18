//! The atlas: coarse cells of about 2 km over the cube faces, each with a height grid sampled once, the
//! flows that start in it and the map of the flows that cross it.
//!
//! Chunks build on worker threads, so the cells live in a shared memory per kind behind a lock, sized by
//! the count of cells of the planet, and each thread keeps the last few it read so that the hot path takes
//! no lock at all.

mod cell;
mod channel;
mod lru;
mod map;
mod mix;
mod near;
mod pits;
mod reach;
mod region;
mod trace;

use std::sync::Arc;

use bevy::math::DVec3;

pub use cell::{Cell, STEPS};
pub use channel::Flowing;
pub use map::FlowMap;
pub use pits::PitMap;
#[cfg(test)]
pub use reach::around;
pub use region::{Region, Sample};
pub use trace::Flow;
#[cfg(test)]
pub use trace::{End, downhill};

use super::Globe;
use lru::Lru;

/// Which memory a cell is kept in, for the per-thread lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    /// Height grids.
    Region,
    /// Maps of the flows that cross a cell.
    Map,
    /// Maps of the craters that reach into a cell.
    Pits,
}

/// The coarse cells of one planet, shared by every copy of its globe.
pub struct Atlas {
    /// A hash of everything the cells depend on, telling this planet's cells from any other's.
    pub(super) print: u64,
    /// Height grids by cell.
    regions: Lru<Region>,
    /// Flows starting in each cell.
    sources: Lru<Vec<Flow>>,
    /// Maps of the flows crossing each cell.
    maps: Lru<FlowMap>,
    /// Maps of the craters reaching into each cell.
    pits: Lru<PitMap>,
}

impl Atlas {
    /// Returns an empty atlas for the planet of fingerprint `print` with `cells` cells per face side: each
    /// memory keeps all the cells of a small planet, and of a large one the [`lru::MOST`] used last.
    pub fn new(print: u64, cells: i32) -> Self {
        let kept = lru::kept(cells);
        Self {
            print,
            regions: Lru::sized(kept),
            sources: Lru::sized(kept),
            maps: Lru::sized(kept),
            pits: Lru::sized(kept),
        }
    }
}

impl Globe {
    /// Returns this globe with a fresh atlas for what it now is.
    pub(super) fn charted(mut self) -> Self {
        let text = format!(
            "{} {} {} {:?} {:?}",
            self.seed, self.radius, self.gravity, self.record, self.setting
        );
        let print = text.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01B3)
        });
        self.atlas = Arc::new(Atlas::new(print, self.cells()));
        self
    }

    /// Returns the cells along one side of a cube face.
    pub(super) fn cells(&self) -> i32 {
        Cell::count(self.radius())
    }

    /// Returns what `read` makes of the cell holding unit direction `dir` and its height grid read there.
    pub(super) fn with_region<R>(&self, dir: DVec3, read: impl FnOnce(Cell, Sample) -> R) -> R {
        let (cell, local) = Cell::locate(dir, self.cells());
        let fetch = || self.atlas.regions.get(cell, || Region::build(self, cell));
        near::near(
            (self.atlas.print, Kind::Region, cell),
            fetch,
            |r: &Region| read(cell, r.sample(local)),
        )
    }

    /// Returns the height of the region grid in unit direction `dir`: the ground before flows and ponds,
    /// smoothed over about 125 m.
    pub(super) fn coarse(&self, dir: DVec3) -> f32 {
        self.with_region(dir, |_, sample| sample.height)
    }

    /// Returns what `read` makes of the flow map of the cell holding unit direction `dir`.
    pub(super) fn with_map<R>(&self, dir: DVec3, read: impl FnOnce(&FlowMap, [f64; 2]) -> R) -> R {
        let (cell, local) = Cell::locate(dir, self.cells());
        let fetch = || self.atlas.maps.get(cell, || FlowMap::build(self, cell));
        near::near((self.atlas.print, Kind::Map, cell), fetch, |m| {
            read(m, local)
        })
    }

    /// Returns the height the craters add at sphere point `p`.
    pub(super) fn pitted(&self, p: DVec3) -> f32 {
        let (cell, local) = Cell::locate(p / self.radius(), self.cells());
        let fetch = || self.atlas.pits.get(cell, || PitMap::build(self, cell));
        near::near((self.atlas.print, Kind::Pits, cell), fetch, |m: &PitMap| {
            m.depth(p, local)
        })
    }

    /// Returns the flows that start in `cell`.
    pub(super) fn sources(&self, cell: Cell) -> Arc<Vec<Flow>> {
        self.atlas.sources.get(cell, || trace::sources(self, cell))
    }
}
