//! Tests that river beds stay bare: the scatter reads a bed the way the terrain mesh paints it, so no plant
//! grows in a river.

use std::sync::Arc;

use bevy::prelude::*;

use super::chunk_center;
use super::tile::Tile;
use crate::world::Surface;
use crate::world::globe::{Face, Globe};

/// Points along one side of the grid a chunk is searched for river bed on, 4 m apart.
const GRID: usize = 16;

/// Returns the chart positions of river bed found on a 4 m grid over the chunk of `tile`.
fn beds(tile: &Tile) -> Vec<Vec2> {
    let centre = chunk_center(tile.coord);
    let at = |i: usize| centre + (Vec2::new((i % GRID) as f32, (i / GRID) as f32) - 7.5) * 4.0;
    (0..GRID * GRID)
        .map(at)
        .filter(|w| tile.terrain.river_at(w.x, w.y))
        .collect()
}

/// Returns up to `want` chunks of `globe` that a river runs through, each with the river bed found on it.
pub(crate) fn river_chunks(globe: &Arc<Globe>, want: usize) -> Vec<(Tile, Vec<Vec2>)> {
    let side = 1i32 << globe.leaf_level();
    let span = -side / 2..side / 2;
    let coords: Vec<IVec2> = span
        .clone()
        .flat_map(|x| span.clone().map(move |y| IVec2::new(x, y)))
        .collect();
    let tiles = Face::ALL
        .into_iter()
        .flat_map(|face| coords.iter().map(move |c| (face, *c)));
    tiles
        .map(|(face, coord)| Tile::new(globe, face, coord))
        .map(|tile| {
            let found = beds(&tile);
            (tile, found)
        })
        .filter(|(_, found)| found.len() >= 4)
        .take(want)
        .collect()
}

/// Returns the home-sized globe of seed 3, whose wet highlands send rivers to the sea.
pub(crate) fn rivers() -> Arc<Globe> {
    Arc::new(Globe::new(3, 6000.0, 22.0))
}

/// On chunks a river runs through, the ground under every river bed point reads as a river, and not one
/// plant of the scatter stands on a river bed.
#[test]
fn nothing_grows_in_a_river() {
    let found = river_chunks(&rivers(), 6);
    assert!(found.len() >= 3, "only {} river chunks", found.len());
    for (tile, beds) in &found {
        let t = &tile.terrain;
        assert!(
            beds.iter()
                .all(|w| t.surface(w.x, w.y) == Surface::River || t.height(w.x, w.y) <= 0.0)
        );
        let centre = chunk_center(tile.coord);
        for (placement, _) in tile.scatter() {
            let w = centre + placement.pos.xz();
            assert!(
                !t.river_at(w.x, w.y),
                "{:?} in a river at {w}",
                placement.plant
            );
        }
    }
}
