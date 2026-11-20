//! Tests of the tiles: their pixels, the tiles a lens looks at, and a coast painted where the
//! ground crosses sea level.

use std::f64::consts::{PI, TAU};

use abstractions_net::leaves::Leaves;
use bevy::math::{DVec3, Vec2};

use super::grid;
use super::tile::{LEVEL_MAX, SHOWN_MAX, SIDE, TileId, level_for, visible};
use super::tile_paint::lit;
use crate::computer::Lens;
use crate::map::explored::grid::{depth_for, leaf_of};
use crate::map::lat_lon;

/// Returns the tile of `level` that holds unit direction `dir`.
pub(super) fn tile_at(dir: DVec3, level: u8) -> TileId {
    let (lat, lon) = lat_lon(dir);
    TileId {
        level,
        col: (lon / TAU * f64::from(TileId::cols(level))) as u32,
        row: ((0.5 - lat / PI) * f64::from(TileId::rows(level))) as u32,
    }
}

/// Returns the tile of `level` at column `col` and row `row`.
fn tile(level: u8, col: u32, row: u32) -> TileId {
    TileId { level, col, row }
}

/// Every pixel of a tile holds the direction of its own middle, on tiles of several levels.
#[test]
fn a_pixel_holds_its_own_middle() {
    for id in [tile(0, 1, 0), tile(3, 9, 5), tile(8, 400, 100)] {
        for (x, y) in [(0, 0), (255, 0), (17, 200), (255, 255), (128, 128)] {
            let dir = id.direction((x as f64 + 0.5, y as f64 + 0.5));
            assert_eq!(id.pixel(dir), Some((x, y)), "{id:?} {x} {y}");
        }
    }
    let far = tile(3, 9, 5).direction((0.5, 0.5));
    assert_eq!(tile(3, 10, 5).pixel(far), None);
}

/// The level follows the zoom, and the tiles a lens looks at hold the point it looks at and
/// stay few.
#[test]
fn a_lens_looks_at_a_few_tiles_that_hold_its_centre() {
    assert!((1..=LEVEL_MAX).contains(&level_for(3.0)));
    assert_eq!(level_for(32.0), LEVEL_MAX);
    assert!(level_for(6.0) <= level_for(12.0));
    let lenses = [
        (3.0, Vec2::ZERO),
        (9.0, Vec2::new(0.3, 0.2)),
        (32.0, Vec2::new(0.99, -0.4)),
    ];
    for (zoom, centre) in lenses {
        let tiles = visible(Lens { zoom, centre }, level_for(zoom));
        assert!(!tiles.is_empty() && tiles.len() <= SHOWN_MAX, "{zoom}");
        let dir = crate::map::from_chart(centre).expect("on the map");
        let holds = tiles.iter().any(|(_, id)| id.pixel(dir).is_some());
        assert!(holds, "no tile holds the centre at zoom {zoom}");
    }
}

/// A lens at the seam looks at tiles east of the last column, which wrap to the first.
#[test]
fn a_lens_at_the_seam_wraps_its_tiles() {
    let lens = Lens {
        zoom: 32.0,
        centre: Vec2::new(0.999, 0.0),
    };
    let cols = i64::from(TileId::cols(5));
    let seam = visible(lens, 5);
    assert!(seam.iter().any(|(col, id)| *col >= cols && id.col < 2));
}

/// The explored leaves light the pixel they fall in and no pixel far from it.
#[test]
fn leaves_light_their_own_pixels() {
    let dir = DVec3::new(0.6, 0.3, -0.7).normalize();
    let mut leaves = Leaves::default();
    leaves.insert(leaf_of(dir, depth_for(6.371e6)));
    let id = tile_at(dir, 8);
    let lit = lit(&leaves, id);
    let (x, y) = id.pixel(dir).expect("inside its tile");
    assert_eq!(lit.len(), 1);
    assert!(
        lit.iter()
            .all(|(a, b)| a.abs_diff(x) <= 2 && b.abs_diff(y) <= 2)
    );
}

/// A tile of level 0 is the picture: its pixel is the cell of the picture grid that holds a
/// point.
#[test]
fn the_tile_pyramid_starts_as_the_picture() {
    let dir = DVec3::new(0.2, 0.5, -0.8).normalize();
    let id = tile_at(dir, 0);
    let (x, y) = id.pixel(dir).expect("inside");
    let cell = grid::cell_of(dir) as usize;
    let (column, row) = (cell % grid::COLS, grid::ROWS - 1 - cell / grid::COLS);
    assert_eq!((column, row), (id.col as usize * SIDE + x, y));
}
