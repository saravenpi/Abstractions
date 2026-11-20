//! Tests of scatter on planets with their life given: a living world grows its own producers, a dead one
//! keeps only its stones, and home plants its life like any other.

use std::sync::Arc;

use super::grow::Flora;
use super::zones::{LUSH, Tile, Zone, chunks};
use super::*;
use crate::life::flora::fixture::{MILD, garden, life, producer};
use crate::life::{Life, Rung};
use crate::universe::{EARTH_RADIUS, STANDARD_GRAVITY};
use crate::world::globe::Globe;
use crate::world::propmesh::grown::{grown, habit};
use Plant::*;

/// Returns six land chunks at face centres of a lush 5,000 m globe of `gravity` carrying `life`.
fn tiles(gravity: f64, life: Option<Life>) -> Vec<Tile> {
    let globe = Arc::new(Globe::new(LUSH, 5000.0, gravity).with_life(life));
    chunks(&globe, Zone::Centre, 15, 6)
}

/// Returns true for the stones every planet keeps.
fn stone(p: &Placement) -> bool {
    matches!(
        p.plant,
        Rock | IceRock | Sandstone | Pebbles | SmoothRock | Boulder
    ) && !grown(p.variant)
}

/// A living planet grows only the architectures of its own producers, each on its carrier, and stones.
#[test]
fn a_living_planet_grows_its_own_producers() {
    let mut seen = [0usize; 8];
    for tile in tiles(9.81, Some(garden(9, &[0, 2, 5], vec![]))) {
        let flora = Flora::of(&tile.terrain, chunk_center(tile.coord)).expect("a flora");
        for (p, _) in tile.scatter() {
            let group = flora.growers.iter().find(|g| g.looks.contains(&p.variant));
            match group {
                Some(g) => {
                    assert_eq!(p.plant, habit(g.arch).carrier);
                    seen[usize::from(g.arch)] += 1;
                }
                None => assert!(
                    stone(&p),
                    "{:?} variant {} on a living planet",
                    p.plant,
                    p.variant
                ),
            }
        }
    }
    assert!(seen[0] > 20 && seen[2] > 100 && seen[5] > 5, "{seen:?}");
    assert_eq!(seen.iter().sum::<usize>(), seen[0] + seen[2] + seen[5]);
}

/// A dead planet grows nothing: only stones lie on it.
#[test]
fn a_dead_planet_grows_nothing() {
    let mut stones = 0;
    for tile in tiles(9.81, None) {
        for (p, _) in tile.scatter() {
            assert!(stone(&p), "{:?} on a dead planet", p.plant);
            stones += 1;
        }
    }
    assert!(stones > 0, "no stone at all");
}

/// A planet of mats and films grows nothing but mats.
#[test]
fn a_planet_of_mats_grows_only_films() {
    let films = life(
        Rung::Mats,
        [0.4, 0.05, 0.02],
        vec![producer(9, 0, 0, MILD), producer(9, 1, 5, MILD)],
        vec![],
    );
    let mut mats = 0;
    for tile in tiles(9.81, Some(films)) {
        for (p, _) in tile.scatter() {
            assert!(
                stone(&p) || p.plant == habit(2).carrier,
                "{:?} on a planet of mats",
                p.plant
            );
            mats += usize::from(!stone(&p));
        }
    }
    assert!(mats > 100, "only {mats} films");
}

/// Trees on a 2 g world stand lower than on a 0.5 g one.
#[test]
fn trees_are_lower_on_a_heavy_world() {
    let mean = |g: f64| {
        let trees: Vec<f32> = tiles(g, Some(garden(9, &[0], vec![])))
            .iter()
            .flat_map(|t| t.scatter())
            .filter(|(p, _)| grown(p.variant))
            .map(|(p, _)| p.scale)
            .collect();
        trees.iter().sum::<f32>() / trees.len().max(1) as f32
    };
    let (heavy, light) = (mean(2.0 * 9.81), mean(0.5 * 9.81));
    assert!(
        heavy > 0.0 && heavy < light,
        "{heavy} at 2 g, {light} at 0.5 g"
    );
}

/// Home with its life given plants its own producers like any living planet, and a home globe whose life
/// was never given keeps the hand-made tables.
#[test]
fn home_plants_its_life() {
    let life = Some(garden(9, &[0, 2], vec![]));
    let given = Arc::new(Globe::new(LUSH, EARTH_RADIUS, STANDARD_GRAVITY).with_life(life));
    let plain = Arc::new(Globe::new(LUSH, EARTH_RADIUS, STANDARD_GRAVITY));
    let (a, b) = (
        chunks(&given, Zone::Centre, 15, 4),
        chunks(&plain, Zone::Centre, 15, 4),
    );
    assert_eq!(a.len(), 4);
    let mut grew = 0;
    for (x, y) in a.iter().zip(&b) {
        let flora = Flora::of(&x.terrain, chunk_center(x.coord)).expect("a flora at home");
        let groups: std::collections::HashSet<u8> = flora.growers.iter().map(|g| g.arch).collect();
        assert!(groups.len() == 2 && flora.growers.len() >= 4, "{flora:?}");
        assert!(Flora::of(&y.terrain, chunk_center(y.coord)).is_none());
        for (p, _) in x.scatter() {
            assert!(
                stone(&p) || grown(p.variant),
                "{:?} of the tables at home",
                p.plant
            );
            grew += usize::from(grown(p.variant));
        }
        assert!(
            y.scatter().iter().any(|(p, _)| !stone(p)),
            "no table plant on a plain home"
        );
    }
    assert!(grew > 20, "only {grew} grown plants at home");
}

/// The same living planet always scatters the same.
#[test]
fn same_life_same_scatter() {
    let (a, b) = (
        tiles(9.81, Some(garden(9, &[1, 3, 4], vec![]))),
        tiles(9.81, Some(garden(9, &[1, 3, 4], vec![]))),
    );
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(x.scatter(), y.scatter());
    }
}
