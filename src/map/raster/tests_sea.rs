//! The colour of the liquid in the body pictures: it is the liquid of the planet, not a fixed
//! blue, and a planet without a sea has none.

use bevy::math::Vec3;

use super::paint::sea_colour;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::Globe;

/// Returns a globe of `class` whose sea covers the `ocean` share of it.
fn globe(class: PlanetClass, ocean: f64) -> Globe {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    Globe::new(3, 3000.0, 9.8).with_record(record)
}

/// The sea of a lava world is molten rock, red and orange and nowhere near blue, at the shore
/// and in the deep.
#[test]
fn a_lava_sea_is_not_blue() {
    let lava = globe(PlanetClass::Lava, 0.4);
    for depth in [-1.0, -20.0, -80.0] {
        let sea = sea_colour(&lava, depth).expect("a lava world has a sea");
        assert!(
            sea.x > sea.y && sea.y >= sea.z && sea.x > 2.0 * sea.z,
            "{depth}: {sea}"
        );
    }
}

/// The sea of an ocean world is water: more blue than red. A world of another class floods
/// with its own colour, not with the water of the home planet.
#[test]
fn each_sea_has_the_colour_of_its_class() {
    let water = sea_colour(&globe(PlanetClass::Ocean, 0.8), -30.0).unwrap();
    assert!(water.z > water.x, "{water}");
    let mut seas: Vec<Vec3> = Vec::new();
    for class in [PlanetClass::Ocean, PlanetClass::Lava, PlanetClass::Ice] {
        seas.push(sea_colour(&globe(class, 0.5), -30.0).unwrap());
    }
    assert!(seas[0] != seas[1] && seas[1] != seas[2] && seas[0] != seas[2]);
}

/// A dry world has no sea colour at any depth, whatever its class.
#[test]
fn a_dry_world_has_no_sea_colour() {
    for class in PlanetClass::ALL {
        let dry = globe(class, 0.0);
        for height in [-100.0, -1.0, 0.0, 5.0] {
            assert_eq!(sea_colour(&dry, height), None, "{class:?} at {height}");
        }
    }
}
