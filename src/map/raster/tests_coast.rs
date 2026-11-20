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

/// Returns true when two colours differ by one byte at most in every channel: the sea shades in
/// steps finer than the samples of its colours.
fn near(first: &[u8; 3], second: &[u8; 3]) -> bool {
    first.iter().zip(second).all(|(a, b)| a.abs_diff(*b) <= 1)
}

/// Returns the pixels of tile `id` where the painted sea and the ground at the middle of the pixel
/// disagree, and how many rows of the tile hold both sea and land.
fn disagreements(globe: &Globe, id: TileId) -> (Vec<(usize, usize)>, usize) {
    let data = paint(globe, id, None);
    let sea = sea_bytes(globe);
    let (mut wrong, mut coasts) = (Vec::new(), 0);
    for y in 0..SIDE {
        let painted = |x: usize| {
            let at = (y * SIDE + x) * BYTES;
            let pixel = [data[at], data[at + 1], data[at + 2]];
            sea.iter().any(|sea| near(sea, &pixel))
        };
        let wet = |x: usize| {
            let dir = id.direction((x as f64 + 0.5, y as f64 + 0.5));
            globe.height(dir) < globe.sea_level()
        };
        wrong.extend((0..SIDE).filter(|x| painted(*x) != wet(*x)).map(|x| (x, y)));
        coasts += usize::from((0..SIDE - 1).any(|x| wet(x) != wet(x + 1)));
    }
    (wrong, coasts)
}

/// A tile 78 km across at a coast paints the sea in the pixels whose middle lies under sea level
/// and the land in the others, so its coastline is within one pixel of where the height of the
/// ground crosses sea level. Sea is told from land by colour, so a thousandth of the pixels
/// may be mistaken where a land colour comes within a byte of a sea colour.
#[test]
fn a_coast_is_painted_within_a_pixel_of_the_crossing() {
    let (globe, on_coast) = coast().expect("a planet with a coast");
    let (wrong, coasts) = disagreements(&globe, tile_at(on_coast, 8));
    assert!(coasts > 0, "no row of the tile crosses the coast");
    let stray = SIDE * SIDE / 1000;
    assert!(
        wrong.len() <= stray,
        "{} pixels, the first {:?}",
        wrong.len(),
        wrong[0]
    );
}
