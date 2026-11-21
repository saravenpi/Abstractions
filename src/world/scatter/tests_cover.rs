//! Tests of the plant cover of living planets: many distinct looks per architecture, patchy cover that varies
//! between planets and within one, barren stretches, and as many plants overall as before.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::cover::Cover;
use super::grow::{Flora, patchiness};
use super::zones::{Tile, Zone, chunks, hold};
use super::*;
use crate::life::flora::fixture::garden;
use crate::world::globe::Globe;
use crate::world::propmesh::grown::grown;

/// Grown plants the eight test planets placed on their test chunks before the cover, big ones and in all.
const BEFORE: (usize, usize) = (513, 3942);

/// Returns `want` land chunks of a 5,000 m globe of seed `seed` carrying one producer of every architecture.
pub(super) fn planet(seed: u64, want: usize) -> Vec<Tile> {
    let all = garden(seed, &[0, 1, 2, 3, 4, 5, 6, 7], vec![]);
    let globe = Arc::new(Globe::new(seed, 5000.0, 9.81).with_life(Some(all)));
    chunks(&globe, Zone::Centre, 15, want)
}

/// Returns the share of the grid of chunk `t` that is more than half covered.
fn covered(t: &Tile) -> f32 {
    let cover = Cover::new(&t.terrain, t.coord, patchiness(t.terrain.seed()), None);
    let corner = t.coord.as_vec2() * CHUNK_SIZE;
    let grid = (0..64)
        .map(|i| corner + (Vec2::new((i % 8) as f32, (i / 8) as f32) + 0.5) * CHUNK_SIZE / 8.0);
    grid.filter(|&w| cover.at(w)[0] > 0.5).count() as f32 / 64.0
}

/// Each architecture a planet grows shows at least five distinct looks: two or three species at four ages.
#[test]
fn every_architecture_shows_many_looks() {
    let _book = hold();
    let mut looks: HashMap<u8, HashSet<u8>> = HashMap::new();
    for seed in [1, 5, 8] {
        for tile in planet(seed, 8) {
            let flora = Flora::of(&tile.terrain, chunk_center(tile.coord)).expect("a flora");
            for (p, _) in tile.scatter().into_iter().filter(|(p, _)| grown(p.variant)) {
                let group = flora.growers.iter().find(|g| g.looks.contains(&p.variant));
                let arch = group.expect("a species of the flora").arch;
                looks.entry(arch).or_default().insert(p.variant);
            }
        }
    }
    let counts: Vec<(u8, usize)> = looks.iter().map(|(a, v)| (*a, v.len())).collect();
    println!("LOOKS per architecture {counts:?}");
    assert!(looks.len() >= 6, "only {counts:?}");
    assert!(counts.iter().all(|&(_, n)| n >= 5), "{counts:?}");
}

/// The covered share of a planet varies with its seed, and within one planet some chunks lie barren while
/// others are covered.
#[test]
fn cover_is_patchy_between_and_within_planets() {
    let shares: Vec<(f32, Vec<f32>)> = (1..=12)
        .map(|seed| {
            let each: Vec<f32> = planet(seed, 24).iter().map(covered).collect();
            (each.iter().sum::<f32>() / each.len().max(1) as f32, each)
        })
        .collect();
    let means: Vec<f32> = shares.iter().map(|s| s.0).collect();
    println!("COVER mean share per seed {means:.2?}");
    let (low, high) = means
        .iter()
        .fold((1f32, 0f32), |(l, h), &m| (l.min(m), h.max(m)));
    assert!(high - low > 0.3, "cover share only spans {low} to {high}");
    let mixed = shares
        .iter()
        .filter(|(_, each)| each.iter().any(|&c| c < 0.1) && each.iter().any(|&c| c > 0.6));
    assert!(
        mixed.count() >= 3,
        "too few planets with both barren and covered chunks"
    );
}

/// Patchiness is a fixed draw of the planet seed between 0 and 1.
#[test]
fn patchiness_is_seeded() {
    let draws: Vec<f32> = (0..64).map(patchiness).collect();
    assert!(draws.iter().all(|p| (0.0..1.0).contains(p)));
    assert_eq!(draws, (0..64).map(patchiness).collect::<Vec<_>>());
    assert!(draws.iter().any(|&p| p < 0.2) && draws.iter().any(|&p| p > 0.8));
}

/// The eight test planets grow about as many plants as before the cover, within 30 percent, big ones and
/// all, so the cover moves plants about rather than adding them.
#[test]
fn cover_keeps_the_plant_count() {
    let _book = hold();
    let (mut big, mut all) = (0, 0);
    for seed in 1..=8 {
        for tile in planet(seed, 8) {
            let s = scatter_chunk(&tile.terrain, tile.coord);
            big += s.props.iter().filter(|p| grown(p.variant)).count();
            all += s
                .props
                .iter()
                .chain(&s.grass)
                .filter(|p| grown(p.variant))
                .count();
        }
    }
    println!("COUNT big {big} all {all}, before {BEFORE:?}");
    let near = |n: usize, was: usize| (n as f32 / was as f32 - 1.0).abs() <= 0.3;
    assert!(
        near(big, BEFORE.0) && near(all, BEFORE.1),
        "big {big} all {all} against {BEFORE:?}"
    );
}

/// A coarse mesh reads the cover of a place as the chunk that holds it does: at the grid points of the cover of a
/// chunk, the spread of the stretch and the carpet of the chunk agree.
#[test]
fn spread_reads_the_cover_of_the_chunks() {
    let _book = hold();
    let tile = planet(3, 1).remove(0);
    let (coord, terrain) = (tile.coord, &tile.terrain);
    let spread = super::turf::Spread::of(terrain, chunk_center(coord)).expect("a spread");
    let corner = coord.as_vec2() * CHUNK_SIZE;
    for k in 0..5 {
        let w = corner + Vec2::new(k as f32, 4.0 - k as f32) * CHUNK_SIZE / 4.0;
        let far = super::air::calm(|| spread.at(terrain, w));
        let near = super::air::calm(|| super::turf::Carpet::of(terrain, coord).map(|c| c.at(w)));
        assert!(
            (near.expect("a carpet") - far).abs() < 1e-4,
            "{near:?} {far}"
        );
    }
}
