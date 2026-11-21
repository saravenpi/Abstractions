//! Tests of the landing pad of the home base: no blade, tuft or prop stands over its square, a margin of one
//! blade past its edge, while the grass around it keeps growing. The blades of the home planets may be few or
//! none, so a lush test planet proves that blades are built and kept outside the square.

use std::sync::Arc;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Cropped;
use super::clear::blocked;
use super::ground::Ground;
use super::tests_tile::{turfed, view};
use super::tile::grow;
use super::tune::{PAD_MARGIN, TILE};
use crate::home::Base;
use crate::universe::Universe;
use crate::world::chunks::planted;
use crate::world::globe::{Pad, Reserve};
use crate::world::mesh::chunk_heights;
use crate::world::rocks::Taken;
use crate::world::scatter::Carpet;
use crate::world::scatter::zones::hold;
use crate::world::{CHUNK_SIZE, Globe, Terrain};

/// Metres from the middle of the pad within which tiles and chunks are checked: past its corner and its margin.
const REACH: f32 = 20.0;

/// Metres past the margin within which blades must still stand.
const BEYOND: f64 = 3.0;

/// Returns the globe of the home planet of `seed` with the ground of its base reserved, as the game does it, the
/// base, and the view of the chart the chunks are drawn on around the pad.
fn base_of(seed: u64) -> (Arc<Globe>, Base, Terrain) {
    let universe = Universe::new(seed);
    let home = universe.home();
    let globe = Arc::new(Globe::of(&universe.planet(home)));
    let base = Base::on(&globe, home);
    let globe = Arc::new(globe.as_ref().clone().reserving(base.reserve()));
    let terrain = Terrain::near(&globe, base.pad);
    (globe, base, terrain)
}

/// Returns the keys of the cells of `size` metres of the chart that lie within [`REACH`] of the pad.
fn cells_near(terrain: &Terrain, pad: DVec3, size: f32) -> Vec<IVec2> {
    let at = terrain.locate(pad);
    let (low, high) = (
        (Vec2::new(at.x, at.z) - REACH) / size,
        (Vec2::new(at.x, at.z) + REACH) / size,
    );
    let (low, high) = (low.floor().as_ivec2(), high.floor().as_ivec2());
    (low.x..=high.x)
        .flat_map(|x| (low.y..=high.y).map(move |y| IVec2::new(x, y)))
        .collect()
}

/// On the home planets of seeds 0 to 9, no blade root of the tiles around the pad lies over its square grown by
/// the margin, and blades still stand within three metres outside it.
#[test]
fn no_blade_stands_on_the_pad() {
    let (mut inside, mut outside, mut total) = (0, 0, 0);
    for seed in 0..10 {
        let (globe, base, terrain) = base_of(seed);
        let none = Cropped::default();
        let clear = |p: DVec3| blocked(p, (&[], &none), &globe);
        for key in cells_near(&terrain, base.pad, TILE) {
            let chunk = (key.as_vec2() * TILE / CHUNK_SIZE).floor().as_ivec2();
            let Some(carpet) = Carpet::of(&terrain, chunk) else {
                continue;
            };
            let ground = Ground::read(&terrain, key);
            let (blades, frame) = grow((&terrain, &ground), key, &carpet, true, &clear);
            for root in blades.roots().iter().step_by(3) {
                let at = frame.origin + frame.rot * Vec3::from_array(*root).as_dvec3();
                total += 1;
                inside += usize::from(globe.on_pad(at, PAD_MARGIN));
                outside += usize::from(globe.on_pad(at, PAD_MARGIN + BEYOND));
            }
        }
    }
    assert_eq!(inside, 0, "{inside} blades stand over the pad");
    assert!(
        outside > 0,
        "no blade stands within {BEYOND} m outside the pad"
    );
}

/// On the same planets, no prop, flower or tuft of the chunks around the pad stands over its square grown by
/// the margin.
#[test]
fn no_plant_stands_on_the_pad() {
    for seed in 0..10 {
        let (globe, base, terrain) = base_of(seed);
        for coord in cells_near(&terrain, base.pad, CHUNK_SIZE) {
            let heights = chunk_heights(&terrain, coord, 1);
            let (scatter, warp) = planted(&terrain, coord, &heights, &Taken::default());
            let frame = warp.frame();
            for p in scatter.props.iter().chain(&scatter.grass) {
                let at = warp.point(Vec3::new(p.pos.x, 0.0, p.pos.z));
                let planet = frame.origin + frame.rot * at.as_dvec3();
                assert!(
                    !globe.on_pad(planet, PAD_MARGIN),
                    "seed {seed}: a plant stands over the pad"
                );
            }
        }
    }
}
