//! Tests of the terrain triangles.

use super::super::style::{SEA_DROP, Style};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::SEA_LEVEL;
use crate::world::lands::Lands;

/// A coarse mesh draws its flat sea 1.5 m under the sea level, and none at all on a dry planet of
/// any class, whose low ground it draws as it is.
#[test]
fn a_coarse_mesh_has_a_sea_only_where_the_planet_has_one() {
    for class in PlanetClass::ALL {
        let lands = |ocean: f64| {
            Lands::of(&PlanetRecord {
                class,
                ocean,
                ..PlanetRecord::TEMPERATE
            })
        };
        assert_eq!(Style::coarse(8.0, lands(0.4)).floor, SEA_LEVEL - SEA_DROP);
        assert_eq!(Style::coarse(8.0, lands(0.0)).floor, f32::NEG_INFINITY);
        assert_eq!(Style::detail(lands(0.4)).floor, f32::NEG_INFINITY);
    }
}
