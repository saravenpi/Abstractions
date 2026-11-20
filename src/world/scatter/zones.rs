//! Test fixtures: the chunks at chosen places of the cube faces of a globe.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use bevy::prelude::*;

pub use super::tile::Tile;
use crate::universe::STANDARD_GRAVITY;
use crate::world::globe::{Face, Globe};

/// Seed of a test globe whose living planet is nearly all covered, so that tests of its producers count
/// plants rather than barren ground.
pub const LUSH: u64 = 1;

/// Held by the tests that write many looks into the book and by those that time scatter, so that the first
/// do not write over the pages the second are timing.
static BOOK: Mutex<()> = Mutex::new(());

/// Returns the hold on the book of grown looks for the length of a test.
pub fn hold() -> MutexGuard<'static, ()> {
    BOOK.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A part of a cube face, by how much the chart is squeezed there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zone {
    /// Around the middle of the face, where the chart is at its widest.
    Centre,
    /// Along the middle of an edge, where one chart axis is squeezed by 30 percent.
    Edge,
    /// Next to a corner, where the chart axes meet at 120 degrees.
    Corner,
}

/// Returns the chunk coordinates of `zone` on a face cut into `side` chunks per side: a block of `reach` chunks.
pub fn coords(zone: Zone, side: i32, reach: i32) -> Vec<IVec2> {
    let (low, high) = (-side / 2, side / 2 - 1);
    let block =
        |x: i32, y: i32| (0..reach).flat_map(move |i| (0..reach).map(move |j| (x + i, y + j)));
    let cells: Vec<(i32, i32)> = match zone {
        Zone::Centre => block(-reach / 2, -reach / 2).collect(),
        Zone::Edge => (-reach..reach)
            .flat_map(|i| [(high, i), (low, i), (i, high), (i, low)])
            .collect(),
        Zone::Corner => [low, high - reach + 1]
            .into_iter()
            .flat_map(|x| [low, high - reach + 1].map(|y| (x, y)))
            .flat_map(|(x, y)| block(x, y))
            .collect(),
    };
    cells.into_iter().map(|(x, y)| IVec2::new(x, y)).collect()
}

/// Returns up to `want` chunks of `zone` over the six faces of `globe` with at least `dry` land points of 25.
pub fn chunks(globe: &Arc<Globe>, zone: Zone, dry: usize, want: usize) -> Vec<Tile> {
    let side = 1i32 << globe.leaf_level();
    let reach = if zone == Zone::Centre { 8 } else { 4 };
    Face::ALL
        .into_iter()
        .flat_map(|face| {
            coords(zone, side, reach)
                .into_iter()
                .map(move |c| (face, c))
        })
        .map(|(face, coord)| Tile::new(globe, face, coord))
        .filter(|spot| spot.dry() >= dry)
        .take(want)
        .collect()
}

/// Returns the chunk at the face centre, at an edge midpoint and next to a corner of a face of a 3,000 m globe.
pub fn trio() -> [Tile; 3] {
    let globe = Arc::new(Globe::new(5, 3000.0, STANDARD_GRAVITY));
    let side = 1i32 << globe.leaf_level();
    let zones = [Zone::Centre, Zone::Edge, Zone::Corner];
    zones.map(|zone| Tile::new(&globe, Face::PosX, coords(zone, side, 1)[0]))
}

/// Returns a 3,000 m globe and a 40,000 m globe, the smallest and the largest planets.
pub fn globes() -> [Arc<Globe>; 2] {
    [3000.0, 40_000.0].map(|radius| Arc::new(Globe::new(5, radius, STANDARD_GRAVITY)))
}

/// Returns, for each zone of each of the two globes, up to `want` chunks with at least `dry` land points of 25.
pub fn tiles(dry: usize, want: usize) -> Vec<(Zone, Tile)> {
    let zones = [Zone::Centre, Zone::Edge, Zone::Corner];
    let each = |globe: Arc<Globe>| {
        zones.map(move |zone| {
            chunks(&globe, zone, dry, want)
                .into_iter()
                .map(move |t| (zone, t))
        })
    };
    globes().into_iter().flat_map(each).flatten().collect()
}
