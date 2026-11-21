//! Which tiles cover the view. The tiles the lens looks at on the level of its zoom are painted
//! after a coarser level that covers the same view in a few tiles, so the view is whole soon
//! after it moves. Each tile in view is drawn by the finest painted tile that holds it: itself,
//! or one of its ancestors drawn larger until the finer one is painted.

use bevy::prelude::*;

use super::tile::{TileId, level_for, visible};
use crate::map::sight::Sight;

/// Levels between the level of the zoom and the coarse level painted first.
const COARSER: u8 = 2;

/// A tile drawn: its column counted without wrapping, the tile and its picture.
pub type Ready = (i64, TileId, Handle<Image>);

/// Returns the coarse level painted first under the level `level`: [`COARSER`] levels up, and
/// never above level 1.
pub fn cover_level(level: u8) -> u8 {
    level.saturating_sub(COARSER).max(1)
}

/// Returns the tile `levels` levels up from tile `id`, whose column counted without wrapping is
/// `col`, with its own column counted the same way.
pub fn ancestor((col, id): (i64, TileId), levels: u8) -> (i64, TileId) {
    let parent = TileId {
        level: id.level - levels,
        col: id.col >> levels,
        row: id.row >> levels,
    };
    (col.div_euclid(1 << levels), parent)
}

/// Returns the tiles to paint for `lens`, in the order they are wanted: the coarse level that
/// covers the view, then the level of its zoom, each nearest the middle of the view first.
pub fn asked(lens: Sight) -> Vec<(i64, TileId)> {
    let level = level_for(lens.zoom);
    let coarse = cover_level(level);
    let mut tiles = match coarse < level {
        true => visible(lens, coarse),
        false => Vec::new(),
    };
    tiles.extend(visible(lens, level));
    tiles
}

/// Returns the tiles to draw for `lens`, the coarsest first so the finer ones lie over them,
/// and whether they cover the whole view. For each tile in view it takes the finest tile among
/// it and its ancestors that `painted` has a picture of.
pub fn shown(lens: Sight, painted: impl Fn(TileId) -> Option<Handle<Image>>) -> (Vec<Ready>, bool) {
    let level = level_for(lens.zoom);
    let mut drawn: Vec<Ready> = Vec::new();
    let mut whole = true;
    for tile in visible(lens, level) {
        let finest = (0..level)
            .map(|up| ancestor(tile, up))
            .find_map(|(col, id)| Some((col, id, painted(id)?)));
        match finest {
            Some(ready) if !drawn.iter().any(|d| (d.0, d.1) == (ready.0, ready.1)) => {
                drawn.push(ready)
            }
            Some(_) => {}
            None => whole = false,
        }
    }
    drawn.sort_by_key(|(_, id, _)| id.level);
    (drawn, whole)
}
