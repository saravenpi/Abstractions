//! Tests of a globe without ground: the smooth deck of a gas giant, and the classes that keep their ground.

use super::Globe;
use super::tests_record::{globe, sphere_dirs};
use crate::universe::{PlanetClass, PlanetRecord};

/// Returns a 150 km gas giant whose record still names a sea.
pub(in crate::world) fn giant() -> Globe {
    let record = PlanetRecord {
        class: PlanetClass::GasGiant,
        ocean: PlanetRecord::BALANCED_OCEAN,
        ..PlanetRecord::TEMPERATE
    };
    Globe::new(5, 150_000.0, 24.0).with_record(record)
}

/// A gas giant reads height 0 everywhere, has no sea whatever its record says, and stops its quadtree
/// six levels above the leaves; every other class keeps its ground and its leaves.
#[test]
fn a_giant_has_a_smooth_deck_and_no_sea() {
    let deck = giant();
    assert!(!deck.has_ground() && !deck.has_sea());
    assert_eq!(deck.sea_level(), f32::NEG_INFINITY);
    assert!(sphere_dirs(2400).all(|dir| deck.height(dir) == 0.0));
    assert_eq!(deck.deepest_level(), deck.leaf_level() - 6);
    for class in PlanetClass::ALL
        .into_iter()
        .filter(|c| *c != PlanetClass::GasGiant)
    {
        let solid = globe(class, 0.4);
        assert!(solid.has_ground() && solid.has_sea(), "{class:?}");
        assert_eq!(solid.deepest_level(), solid.leaf_level());
        assert!(sphere_dirs(200).any(|dir| solid.height(dir) != 0.0));
    }
}
