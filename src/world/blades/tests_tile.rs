//! Tests of the tiles of blades: they stand where ground cover grows and nowhere else, stay clear of props,
//! cost little to build, and stay under the blade budget of the rings.

use std::sync::Arc;
use std::time::Instant;

use bevy::prelude::*;

use super::tile::{Ground, grow};
use super::tune::{DENSE, FAR, NEAR, SPARSE, TILE};
use crate::life::flora::fixture::garden;
use crate::world::Terrain;
use crate::world::globe::{Face, Globe};
use crate::world::scatter::Carpet;
use crate::world::scatter::zones::{LUSH, hold};

/// Returns the view of the middle face of the lush test globe whose life holds the producers `archs`.
fn view(archs: &[u8]) -> Terrain {
    let life = garden(LUSH, archs, vec![]);
    let globe = Arc::new(Globe::new(LUSH, 5000.0, 9.81).with_life(Some(life)));
    Terrain::on(globe.clone(), globe.face_chart(Face::ALL[0]))
}

/// Returns the first tile near the middle of the face with turf, its carpet and how many dense blades it grows.
fn turfed(terrain: &Terrain) -> Option<(IVec2, Carpet, usize)> {
    let keys = (-40..40).flat_map(|x| (-40..40).map(move |y| IVec2::new(x * 3, y * 3)));
    keys.filter_map(|key| {
        let chunk = (key.as_vec2() * TILE / crate::world::CHUNK_SIZE)
            .floor()
            .as_ivec2();
        let carpet = Carpet::of(terrain, chunk)?;
        let ground = Ground::read(terrain, key);
        let (blades, _) = grow((terrain, &ground), key, &carpet, true, &|_| false);
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

/// The ground a prop takes stays bare, and the sparse blades are the first of the dense ones.
#[test]
fn props_clear_their_ground() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, dense) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let (bare, _) = grow((&terrain, &ground), key, &carpet, true, &|_| true);
    assert_eq!(bare.len(), 0);
    let (sparse, _) = grow((&terrain, &ground), key, &carpet, false, &|_| false);
    let share = sparse.len() as f32 / dense as f32;
    assert!(
        (share - SPARSE / DENSE).abs() < 0.06,
        "sparse share {share}"
    );
}

/// A dense tile builds in under 2 ms, and every tile of the rings holds fewer than 150,000 blades in all:
/// 450,000 vertices of a three-vertex blade, well inside the GPU budget of the blades.
#[test]
fn blades_are_cheap() {
    let _book = hold();
    let terrain = view(&[0, 2, 5]);
    let (key, carpet, _) = turfed(&terrain).expect("a turfed tile");
    let ground = Ground::read(&terrain, key);
    let once = || {
        let start = Instant::now();
        let ground = Ground::read(&terrain, key);
        let n = grow((&terrain, &ground), key, &carpet, true, &|_| false)
            .0
            .len();
        (start.elapsed().as_secs_f64(), n)
    };
    let best = (0..5).map(|_| once().0).fold(f64::MAX, f64::min);
    println!(
        "dense tile of {} blades: {:.2} ms",
        grow((&terrain, &ground), key, &carpet, true, &|_| false)
            .0
            .len(),
        best * 1e3
    );
    assert!(best < 2e-3, "a dense tile took {:.2} ms", best * 1e3);
    let reach = ((FAR + TILE) / TILE).ceil() as i32;
    let centre = |x: i32, y: i32| (Vec2::new(x as f32, y as f32) + 0.5) * TILE;
    let tiles = (-reach..=reach).flat_map(|x| (-reach..=reach).map(move |y| centre(x, y).length()));
    let blades: f32 = tiles
        .filter(|d| *d < FAR + TILE)
        .map(|d| TILE * TILE * if d < NEAR + TILE { DENSE } else { SPARSE })
        .sum();
    assert!(blades < 150_000.0, "{blades} blades in the rings");
}
