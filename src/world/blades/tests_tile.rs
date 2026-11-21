//! Tests of the tiles of blades: they stand where ground cover grows and nowhere else, stay clear of props,
//! split into bands that make up the dense blades, and cost little to build.

use std::sync::Arc;
use std::time::Instant;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::Cropped;
use super::clear::{reaching, taken};
use super::colour::tint;
use super::fade::{ALL, ranks};
use super::ground::Ground;
use super::tile::grow;
use super::tune::{BANDS, TILE};
use crate::life::flora::fixture::garden;
use crate::world::Terrain;
use crate::world::globe::{Face, Globe};
use crate::world::scatter::Carpet;
use crate::world::scatter::zones::{LUSH, hold};

/// Returns the view of the middle face of the lush test globe whose life holds the producers `archs`.
pub(super) fn view(archs: &[u8]) -> Terrain {
    let life = garden(LUSH, archs, vec![]);
    let globe = Arc::new(Globe::new(LUSH, 5000.0, 9.81).with_life(Some(life)));
    Terrain::on(globe.clone(), globe.face_chart(Face::ALL[0]))
}

/// Returns the first tile near the middle of the face with turf, its carpet and how many dense blades it grows.
pub(super) fn turfed(terrain: &Terrain) -> Option<(IVec2, Carpet, usize)> {
    let keys = (-40..40).flat_map(|x| (-40..40).map(move |y| IVec2::new(x * 3, y * 3)));
    keys.filter_map(|key| {
        let chunk = (key.as_vec2() * TILE / crate::world::CHUNK_SIZE)
            .floor()
            .as_ivec2();
        let carpet = Carpet::of(terrain, chunk)?;
        let ground = Ground::read(terrain, key);
        let (blades, _) = grow((terrain, &ground), key, &carpet, ALL, &|_| false);
        Some((key, carpet, blades.len()))
    })
    .find(|(_, _, n)| *n > 1000)
}

/// A planet whose producers are trees and stalks has no turf and grows no blade; one with mats grows them.
#[test]
fn blades_only_on_ground_cover() {
    let _book = hold();
    let trees = view(&[0, 5]);
    let chunks = (-8..8).flat_map(|x| (-8..8).map(move |y| IVec2::new(x, y)));
    assert!(chunks.clone().all(|c| Carpet::of(&trees, c).is_none()));
    let meadow = view(&[0, 2, 5]);
    assert!(chunks.clone().any(|c| Carpet::of(&meadow, c).is_some()));
    assert!(
        turfed(&meadow).is_some(),
        "no tile of the mat planet grows blades"
    );
}

/// The ground a prop takes stays bare, each band holds its share of the dense blades, and the bands together
/// hold every dense blade, root for root.
#[test]
fn props_clear_their_ground() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, dense) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let (bare, _) = grow((&terrain, &ground), key, &carpet, ALL, &|_| true);
    assert_eq!(bare.len(), 0);
    let mut roots = Vec::new();
    for band in 0..BANDS.len() {
        let (blades, _) = grow((&terrain, &ground), key, &carpet, ranks(band), &|_| false);
        let share = blades.len() as f32 / dense as f32;
        let wanted = ranks(band).end - ranks(band).start;
        assert!((share - wanted).abs() < 0.06, "band {band} share {share}");
        roots.extend_from_slice(blades.roots());
    }
    let (all, _) = grow((&terrain, &ground), key, &carpet, ALL, &|_| false);
    assert_eq!(roots, all.roots());
}

/// A dense tile builds in under 2 ms.
#[test]
fn blades_are_cheap() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, _) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let once = || {
        let start = Instant::now();
        let ground = Ground::read(&terrain, key);
        let n = grow((&terrain, &ground), key, &carpet, ALL, &|_| false)
            .0
            .len();
        (start.elapsed().as_secs_f64(), n)
    };
    let best = (0..5).map(|_| once().0).fold(f64::MAX, f64::min);
    println!(
        "dense tile of {} blades: {:.2} ms",
        grow((&terrain, &ground), key, &carpet, ALL, &|_| false)
            .0
            .len(),
        best * 1e3
    );
    assert!(best < 2e-3, "a dense tile took {:.2} ms", best * 1e3);
}

/// A prop that cannot reach the tile changes nothing: the blades grown clear of the footprints that reach
/// the tile are those grown clear of every footprint, with a prop inside the tile, on its border and far off.
#[test]
fn reaching_keeps_the_blades() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, _) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let sphere = ground.sphere(&terrain);
    let corner = key.as_vec2() * TILE;
    let on = |x: f32, z: f32| terrain.embed(x, terrain.height(x, z), z);
    let props = [
        (on(corner.x + TILE * 0.5, corner.y + TILE * 0.5), 1.5),
        (on(corner.x, corner.y + TILE * 0.5), 0.6),
        (sphere.0 + DVec3::new(300.0, 0.0, 0.0), 2.0),
    ];
    let kept = reaching(&props, sphere);
    assert_eq!(kept.len(), 2, "the far prop is dropped, the others stay");
    let radius = terrain.globe().radius();
    let crops = Cropped::default();
    let all = |p| taken(p, &props, &crops, radius);
    let some = |p| taken(p, &kept, &crops, radius);
    let free = grow((&terrain, &ground), key, &carpet, ALL, &|_| false).0;
    let (full, _) = grow((&terrain, &ground), key, &carpet, ALL, &all);
    let (filtered, _) = grow((&terrain, &ground), key, &carpet, ALL, &some);
    assert!(full.len() < free.len(), "the props clear no blade");
    assert_eq!(full.roots(), filtered.roots());
}

/// The baked carpet noise stays in 0 to 1 and repeats every 256 metres, as the shader noise does.
#[test]
fn the_tint_repeats() {
    let spots = (0..50).map(|n| Vec2::new(n as f32 * 5.3, n as f32 * 2.9 + 0.4));
    for p in spots {
        let (share, speckle) = tint(p);
        assert!((0.0..=1.0).contains(&share) && (0.0..=1.0).contains(&speckle));
        let (again, same) = tint(p + Vec2::new(256.0, 256.0));
        assert!((share - again).abs() < 1e-3 && speckle == same);
    }
}
