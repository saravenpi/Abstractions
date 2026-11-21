//! Tests of what grows on each class of planet: nothing on molten or bare rock, desert plants on a desert,
//! cold plants on ice, and no sea life without a living sea.

use std::sync::Arc;

use super::rules::underwater;
use super::tile::Tile;
use super::zones::{Zone, chunks};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::globe::Globe;
use crate::world::propmesh::Plant;

use Plant::*;

/// The plants of the desert tables.
const DESERT: [Plant; 7] = [
    Saguaro,
    RoundCactus,
    DeadBush,
    DeadTree,
    Sandstone,
    Grass,
    Pebbles,
];

/// The plants of the taiga tables on land.
const COLD: [Plant; 8] = [
    SnowPine, Pine, DeadTree, DeadBush, IceRock, Grass, Fern, Pebbles,
];

/// Returns a 6 km globe of seed 5 with the home record changed into `class` and an `ocean` share.
fn globe(class: PlanetClass, ocean: f64) -> Arc<Globe> {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    Arc::new(Globe::new(5, 6000.0, 22.0).with_record(record))
}

/// Returns the home globe and up to `want` of its chunks with `dry` land points of 25.
fn home_tiles(dry: std::ops::Range<usize>, want: usize) -> (Arc<Globe>, Vec<Tile>) {
    let home = Arc::new(Globe::new(5, 6000.0, 22.0));
    let all = chunks(&home, Zone::Centre, 0, 400).into_iter();
    let tiles = all.filter(|t| dry.contains(&t.dry())).take(want).collect();
    (home, tiles)
}

/// Returns everything that stands on the chunks of `tiles` once they lie on `globe`.
fn plants(globe: &Arc<Globe>, tiles: &[Tile]) -> Vec<Plant> {
    let on = tiles.iter().map(|t| Tile::new(globe, t.face, t.coord));
    on.flat_map(|t| t.scatter()).map(|(p, _)| p.plant).collect()
}

/// A globe given the home record scatters exactly what the globe built without one does.
#[test]
fn the_home_scatter_is_unchanged() {
    let (home, tiles) = home_tiles(5..26, 8);
    let same = globe(PlanetClass::Island, PlanetRecord::BALANCED_OCEAN);
    assert_eq!(tiles.len(), 8);
    for tile in &tiles {
        let again = Tile::new(&same, tile.face, tile.coord);
        assert_eq!(tile.scatter(), again.scatter());
    }
    assert!(plants(&home, &tiles).len() > 800);
}

/// Nothing is scattered on molten or bare rock, with or without a sea.
#[test]
fn lifeless_classes_grow_nothing() {
    let (_, tiles) = home_tiles(5..26, 8);
    for class in [PlanetClass::Lava, PlanetClass::Barren] {
        for ocean in [0.0, 0.3, PlanetRecord::BALANCED_OCEAN] {
            let found = plants(&globe(class, ocean), &tiles);
            assert!(found.is_empty(), "{class:?} grows {found:?}");
        }
    }
}

/// A desert world grows only desert plants and an ice world only cold ones, both thinner than home.
#[test]
fn desert_and_ice_grow_their_own_sparse_plants() {
    let (home, tiles) = home_tiles(22..26, 8);
    let lush = plants(&home, &tiles).len();
    let cases = [
        (PlanetClass::Desert, &DESERT[..]),
        (PlanetClass::Ice, &COLD[..]),
    ];
    for (class, allowed) in cases {
        let found = plants(&globe(class, 0.0), &tiles);
        let odd = found.iter().find(|p| !allowed.contains(p));
        assert!(odd.is_none(), "{class:?} grows {odd:?}");
        assert!(!found.is_empty(), "{class:?} grows nothing");
        assert!(
            found.len() * 2 < lush,
            "{class:?}: {} of {lush}",
            found.len()
        );
    }
}

/// A dry world of any class has no sea life, no shore prop and no pond plant, on chunks that
/// are coast and sea at home.
#[test]
fn a_dry_world_has_no_sea_life() {
    let (home, tiles) = home_tiles(0..20, 14);
    let wet = plants(&home, &tiles);
    assert!(wet.iter().any(|p| underwater(*p)), "the fixture has no sea");
    for class in PlanetClass::ALL {
        let found = plants(&globe(class, 0.0), &tiles);
        let odd = found.iter().find(|p| underwater(**p) || p.pond());
        assert!(odd.is_none(), "dry {class:?} has {odd:?}");
    }
}

/// A sea that is not the water of home has a bare floor: nothing of the sea stands under it.
#[test]
fn only_a_living_sea_has_sea_life() {
    let (_, tiles) = home_tiles(0..20, 14);
    for class in PlanetClass::ALL {
        let found = plants(&globe(class, PlanetRecord::BALANCED_OCEAN), &tiles);
        let alive = found.iter().any(|p| underwater(*p));
        let water = matches!(class, PlanetClass::Ocean | PlanetClass::Island);
        assert_eq!(alive, water, "{class:?}");
    }
}
