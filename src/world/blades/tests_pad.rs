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
use crate::world::lands::{Verdure, thrive::climate_at};
use crate::world::mesh::chunk_heights;
use crate::world::rocks::Taken;
use crate::world::scatter::zones::hold;
use crate::world::scatter::{Carpet, Flora, turf_of};
use crate::world::{CHUNK_SIZE, Globe, Terrain};

/// Metres from the middle of the pad within which tiles and chunks are checked: past its corner and its margin.
const REACH: f32 = 20.0;

/// Metres past the margin within which blades must still stand.
const BEYOND: f64 = 3.0;

/// Returns the home planet of `seed` as the game streams it, with its life, with the ground of its base reserved
/// when `reserved`, the base, and the view of the chart the chunks are drawn on around the pad.
fn base_of(seed: u64, reserved: bool) -> (Arc<Globe>, Base, Terrain) {
    let universe = Universe::new(seed);
    let home = universe.home();
    let spec = universe.planet(home);
    let life = crate::life::of(&spec, &universe.system(home.star).star);
    let globe = Arc::new(Globe::of(&spec).with_life(life));
    let base = Base::on(&globe, home);
    let globe = match reserved {
        true => Arc::new(globe.as_ref().clone().reserving(base.reserve())),
        false => globe,
    };
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

/// Prints the biome and the local temperature at the pad of `base`, the colours of the turf there and the mean
/// colour the ground bakes.
fn describe(seed: u64, terrain: &Terrain, base: &Base) {
    let at = terrain.locate(base.pad);
    let at = Vec2::new(at.x, at.z);
    let air = climate_at(terrain, at).map(|c| c.temperature);
    let turf = Flora::of(terrain, at).and_then(|flora| turf_of(&flora));
    let seen = Verdure::of(terrain, at);
    println!(
        "PAD seed {seed}: biome {:?}, {air:?} K, turf {:?}, baked mean {seen:?}",
        terrain.biome(at.x, at.y),
        turf.map(|t| t.colours)
    );
}

/// Counts the blades of the tiles around the pad of `base` on `globe`: tiles with a turf, tiles without, blades
/// built, blades within [`BEYOND`] metres outside the pad square and blades over it.
fn count(globe: &Arc<Globe>, base: &Base, terrain: &Terrain) -> [usize; 5] {
    let mut n = [0; 5];
    let none = Cropped::default();
    let clear = |p: DVec3| blocked(p, (&[], &none), globe);
    for key in cells_near(terrain, base.pad, TILE) {
        let chunk = (key.as_vec2() * TILE / CHUNK_SIZE).floor().as_ivec2();
        let Some(carpet) = Carpet::of(terrain, chunk) else {
            n[1] += 1;
            continue;
        };
        n[0] += 1;
        let ground = Ground::read(terrain, key);
        let (blades, frame) = grow((terrain, &ground), key, &carpet, true, &clear);
        for root in blades.roots().iter().step_by(3) {
            let at = frame.origin + frame.rot * Vec3::from_array(*root).as_dvec3();
            n[2] += 1;
            n[3] += usize::from(globe.on_pad(at, PAD_MARGIN + BEYOND));
            n[4] += usize::from(globe.on_pad(at, PAD_MARGIN));
        }
    }
    n
}

/// On the home planets of seeds 0 to 9, no blade root of the tiles around the pad lies over its square grown by
/// the margin, and blades still stand within three metres outside it. The same tiles are counted on the globe
/// without the reserve, which tells a missing turf from a cleared one.
#[test]
fn no_blade_stands_on_the_pad() {
    let _book = hold();
    let (mut over, mut near) = (0, 0);
    for seed in 0..10 {
        let (open, base, terrain) = base_of(seed, false);
        describe(seed, &terrain, &base);
        let free = count(&open, &base, &terrain);
        let (shut, base, terrain) = base_of(seed, true);
        let kept = count(&shut, &base, &terrain);
        println!(
            "PAD seed {seed}: [turf tiles, bare tiles, blades, near, over] open {free:?}, reserved {kept:?}"
        );
        (over, near) = (over + kept[4], near + kept[3]);
    }
    assert_eq!(over, 0, "{over} blades stand over the pad");
    assert!(
        near > 0,
        "no blade stands within {BEYOND} m outside the pad"
    );
}

/// Returns the planet position of the root of every blade of the dense tile `key` of `terrain` over `ground`
/// and `carpet`, but those `clear` says are taken.
fn roots(
    (terrain, ground, key, carpet): (&Terrain, &Ground, IVec2, &Carpet),
    clear: &dyn Fn(DVec3) -> bool,
) -> Vec<DVec3> {
    let (blades, frame) = grow((terrain, ground), key, carpet, true, clear);
    let at = |root: &[f32; 3]| frame.origin + frame.rot * Vec3::from_array(*root).as_dvec3();
    blades.roots().iter().step_by(3).map(at).collect()
}

/// Returns `globe` with a pad of two metres a side kept bare around planet position `at`.
fn padded(globe: &Arc<Globe>, at: DVec3) -> Arc<Globe> {
    let up = at.normalize();
    let heading = up.cross(DVec3::Y).normalize_or(DVec3::X);
    let square = Pad {
        centre: at,
        up,
        heading,
        half: 1.0,
    };
    let reserve = Reserve {
        centre: at,
        up,
        radius: 0.0,
        pad: Some(square),
    };
    Arc::new(globe.as_ref().clone().reserving(reserve))
}

/// Where blades certainly grow, a pad of two metres a side bares its square and a margin of one blade and nothing
/// else: the same tile grows blades there without the pad, none with it, and the same ones everywhere else.
#[test]
fn the_pad_bares_only_its_square() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, _) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let (_, frame) = grow((&terrain, &ground), key, &carpet, true, &|_| false);
    let shut = padded(terrain.globe(), frame.origin);
    let none = Cropped::default();
    let tile = (&terrain, &ground, key, &carpet);
    let all = roots(tile, &|_| false);
    let kept = roots(tile, &|p| blocked(p, (&[], &none), &shut));
    let under = all.iter().filter(|p| shut.on_pad(**p, PAD_MARGIN)).count();
    println!(
        "PAD tile blades {}, under the pad {under}, kept {}",
        all.len(),
        kept.len()
    );
    assert!(under > 0, "the tile grows no blade where the pad is put");
    assert!(kept.iter().all(|p| !shut.on_pad(*p, PAD_MARGIN)));
    assert_eq!(
        kept.len(),
        all.len() - under,
        "blades were lost outside the pad"
    );
}

/// On the same planets, no prop, flower or tuft of the chunks around the pad stands over its square grown by
/// the margin.
#[test]
fn no_plant_stands_on_the_pad() {
    for seed in 0..10 {
        let (globe, base, terrain) = base_of(seed, true);
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
