//! Test of a coast in a tile: painted where the ground crosses sea level.

use std::collections::HashSet;

use bevy::math::DVec3;

use super::BYTES;
use super::paint::sea_colour;
use super::tests_tile::tile_at;
use super::tile::{SIDE, TileId};
use super::tile_paint::paint;
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

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

/// Returns, for each row of tile `id` that crosses a coast, where the ground crosses sea level
/// in pixels from the left edge and where the painted sea ends.
fn crossings(globe: &Globe, id: TileId) -> Vec<(f64, f64)> {
    let data = paint(globe, id, None);
    let sea = sea_bytes(globe);
    let mut found = Vec::new();
    for y in 0..SIDE {
        let painted = |x: usize| {
            let at = (y * SIDE + x) * BYTES;
            sea.contains(&[data[at], data[at + 1], data[at + 2]])
        };
        let wet = |x: f64| globe.height(id.direction((x, y as f64 + 0.5))) < globe.sea_level();
        let fine = |sub: usize| (sub as f64 + 0.5) / 4.0;
        let truth = (0..SIDE * 4 - 1).find(|sub| wet(fine(*sub)) != wet(fine(sub + 1)));
        let shown = (0..SIDE - 1).find(|x| painted(*x) != painted(x + 1));
        if let (Some(truth), Some(shown)) = (truth, shown) {
            found.push(((truth + 1) as f64 / 4.0, (shown + 1) as f64));
        }
    }
    found
}

/// A tile 78 km across at a coast shows the coastline within one pixel of where the height of
/// the ground crosses sea level.
#[test]
fn a_coast_is_painted_within_a_pixel_of_the_crossing() {
    let (globe, on_coast) = coast().expect("a planet with a coast");
    let found = crossings(&globe, tile_at(on_coast, 8));
    assert!(!found.is_empty(), "no row of the tile crosses the coast");
    for (truth, shown) in found {
        assert!((truth - shown).abs() <= 1.0, "{truth} against {shown}");
    }
}
