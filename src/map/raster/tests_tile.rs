//! Tests of the tiles: their pixels, the tiles a lens looks at, and a coast painted where the
//! ground crosses sea level.

use std::collections::HashSet;
use std::f64::consts::{PI, TAU};

use abstractions_net::leaves::Leaves;
use bevy::math::{DVec3, Vec2};

use super::paint::sea_colour;
use super::tile::{LEVEL_MAX, SHOWN_MAX, SIDE, TileId, level_for, visible};
use super::tile_paint::{lit, paint};
use super::{BYTES, grid};
use crate::computer::Lens;
use crate::map::explored::grid::{depth_for, leaf_of};
use crate::map::lat_lon;
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

/// Every pixel of a tile holds the direction of its own middle, on tiles of several levels.
#[test]
fn a_pixel_holds_its_own_middle() {
    for id in [
        TileId {
            level: 0,
            col: 1,
            row: 0,
        },
        TileId {
            level: 3,
            col: 9,
            row: 5,
        },
        TileId {
            level: 8,
            col: 400,
            row: 100,
        },
    ] {
        for (x, y) in [(0, 0), (255, 0), (17, 200), (255, 255), (128, 128)] {
            let dir = id.direction((x as f64 + 0.5, y as f64 + 0.5));
            assert_eq!(id.pixel(dir), Some((x, y)), "{id:?} {x} {y}");
        }
    }
    let far = TileId {
        level: 3,
        col: 9,
        row: 5,
    }
    .direction((0.5, 0.5));
    assert_eq!(
        TileId {
            level: 3,
            col: 10,
            row: 5
        }
        .pixel(far),
        None
    );
}

/// The level follows the zoom, and the tiles a lens looks at hold the point it looks at, wrap
/// over the seam, and stay few.
#[test]
fn a_lens_looks_at_a_few_tiles_that_hold_its_centre() {
    assert!((1..=LEVEL_MAX).contains(&level_for(3.0)));
    assert_eq!(level_for(32.0), LEVEL_MAX);
    assert!(level_for(6.0) <= level_for(12.0));
    for (zoom, centre) in [
        (3.0, Vec2::ZERO),
        (9.0, Vec2::new(0.3, 0.2)),
        (32.0, Vec2::new(0.99, -0.4)),
    ] {
        let lens = Lens { zoom, centre };
        let level = level_for(zoom);
        let tiles = visible(lens, level);
        assert!(
            !tiles.is_empty() && tiles.len() <= SHOWN_MAX,
            "{zoom}: {}",
            tiles.len()
        );
        let dir = crate::map::from_chart(centre).expect("on the map");
        let id = tiles
            .iter()
            .map(|(_, id)| *id)
            .find(|id| id.pixel(dir).is_some());
        assert!(id.is_some(), "no tile holds the centre at zoom {zoom}");
    }
    let seam = visible(
        Lens {
            zoom: 32.0,
            centre: Vec2::new(0.999, 0.0),
        },
        5,
    );
    let cols = TileId::cols(5);
    assert!(
        seam.iter()
            .any(|(col, id)| *col >= i64::from(cols) && id.col < 2)
    );
}

/// The explored leaves light the pixels they fall in and no pixel far from them.
#[test]
fn leaves_light_their_own_pixels() {
    let dir = DVec3::new(0.6, 0.3, -0.7).normalize();
    let mut leaves = Leaves::default();
    leaves.insert(leaf_of(dir, depth_for(6.371e6)));
    let (lat, lon) = lat_lon(dir);
    let level = 8u8;
    let id = TileId {
        level,
        col: (lon / TAU * f64::from(TileId::cols(level))) as u32,
        row: ((0.5 - lat / PI) * f64::from(TileId::rows(level))) as u32,
    };
    let lit = lit(&leaves, id);
    let (x, y) = id.pixel(dir).expect("inside its tile");
    assert_eq!(lit.len(), 1);
    assert!(
        lit.iter()
            .all(|(a, b)| a.abs_diff(x) <= 2 && b.abs_diff(y) <= 2)
    );
}

/// Returns a planet with ground and a sea of the first system, and a direction on its coast.
fn coast() -> Option<(Globe, DVec3)> {
    let universe = Universe::new(7);
    let home = universe.home();
    for index in 0..12 {
        let globe = Globe::of(&universe.planet(BodyId { index, ..home }));
        if !globe.has_ground() {
            continue;
        }
        let wet = |dir: DVec3| globe.height(dir) < globe.sea_level();
        for lat in [0.0, 0.2, -0.2, 0.4, -0.4, 0.6] {
            let at = |step: i32| crate::computer::direction(f64::from(step) * 0.002, lat);
            if let Some(step) = (0..3000).find(|step| wet(at(*step)) != wet(at(step + 1))) {
                return Some((globe, at(step)));
            }
        }
    }
    None
}

/// Returns the bytes of the sea colours of `globe`, from the shore to the deep.
fn sea_bytes(globe: &Globe) -> HashSet<[u8; 3]> {
    let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    (0..=200)
        .filter_map(|step| sea_colour(globe, -(step as f32) * 0.25))
        .map(|sea| [byte(sea.x), byte(sea.y), byte(sea.z)])
        .collect()
}

/// A tile 78 km across at a coast shows the coastline within one pixel of where the height of the
/// ground crosses sea level.
#[test]
fn a_coast_is_painted_within_a_pixel_of_the_crossing() {
    let (globe, on_coast) = coast().expect("a planet with a coast");
    let (lat, lon) = lat_lon(on_coast);
    let level = 8u8;
    let id = TileId {
        level,
        col: (lon / TAU * f64::from(TileId::cols(level))) as u32,
        row: ((0.5 - lat / PI) * f64::from(TileId::rows(level))) as u32,
    };
    let data = paint(&globe, id, None);
    let sea = sea_bytes(&globe);
    let mut compared = 0;
    for y in 0..SIDE {
        let painted_sea = |x: usize| {
            let at = (y * SIDE + x) * BYTES;
            sea.contains(&[data[at], data[at + 1], data[at + 2]])
        };
        let wet = |x: f64| {
            let dir = id.direction((x, y as f64 + 0.5));
            globe.height(dir) < globe.sea_level()
        };
        let fine = |sub: usize| (sub as f64 + 0.5) / 4.0;
        let truth = (0..SIDE * 4 - 1).find(|sub| wet(fine(*sub)) != wet(fine(sub + 1)));
        let shown = (0..SIDE - 1).find(|x| painted_sea(*x) != painted_sea(x + 1));
        if let (Some(truth), Some(shown)) = (truth, shown) {
            compared += 1;
            let (truth, shown) = ((truth + 1) as f64 / 4.0, (shown + 1) as f64);
            assert!(
                (truth - shown).abs() <= 1.0,
                "row {y}: {truth} against {shown}"
            );
        }
    }
    assert!(compared > 0, "no row of the tile crosses the coast");
}

/// A tile of level 0 is the picture: its pixel is the cell of the picture grid that holds a point.
#[test]
fn the_tile_pyramid_starts_as_the_picture() {
    let dir = DVec3::new(0.2, 0.5, -0.8).normalize();
    let (_, lon) = lat_lon(dir);
    let id = TileId {
        level: 0,
        col: (lon / PI) as u32,
        row: 0,
    };
    let (x, y) = id.pixel(dir).expect("inside");
    let cell = grid::cell_of(dir) as usize;
    let (column, row) = (cell % grid::COLS, grid::ROWS - 1 - cell / grid::COLS);
    assert_eq!((column, row), (id.col as usize * SIDE + x, y));
}
