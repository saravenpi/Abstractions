//! Tests of what makes planets differ: the same rules give dead, scarred plates to an old small dry world
//! and moving plates, ridges and volcanoes to a young or a wet one, from their own data alone.

use bevy::math::DVec3;

use super::tests::{EARTH, earth};
use super::*;
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::world::globe::tests_record::sphere_dirs;

/// Returns the plates of a world as small, light, dry and airless as Mars, 4.5 billion years old.
fn old_small() -> Plates {
    let record = PlanetRecord {
        class: PlanetClass::Barren,
        pressure: 0.006,
        temperature: 210.0,
        water: Water::None,
        ocean: 0.0,
        ..PlanetRecord::TEMPERATE
    };
    let body = [3.39e6, 3.71];
    Plates::new(5, body[0], Interior::of(5, body, &record, 4.5))
}

/// Returns the plates of a world a little larger than the Earth, half a billion years old, its surface at
/// 900 K under seas of molten rock.
fn young_hot() -> Plates {
    let record = PlanetRecord {
        class: PlanetClass::Lava,
        temperature: 900.0,
        water: Water::Steam,
        ocean: 0.3,
        ..PlanetRecord::TEMPERATE
    };
    let body = [7.0e6, 11.0];
    Plates::new(5, body[0], Interior::of(5, body, &record, 0.5))
}

/// Returns the lowest and the highest of `read` over 20,000 samples of `plates`.
fn span(plates: &Plates, read: impl Fn(&Sample) -> f32) -> (f32, f32) {
    sphere_dirs(20_000)
        .map(|d| read(&plates.sample(d)))
        .fold((f32::MAX, f32::MIN), |(low, high), v| {
            (low.min(v), high.max(v))
        })
}

/// The interior follows the mass, the age, the water and the heat: the Earth still moves, a Mars has
/// stopped, a young molten world is at its most active, and a hot surface alone keeps an old crust moving.
#[test]
fn the_interior_follows_the_planet() {
    let (home, dead, hot) = (earth(5), old_small(), young_hot());
    let (home, dead, hot) = (home.interior(), dead.interior(), hot.interior());
    assert!((0.4..0.7).contains(&home.activity), "{home:?}");
    assert_eq!(home.mobility, home.activity);
    assert!(home.scars < 0.1 && home.lowland == PlanetRecord::BALANCED_OCEAN);
    assert_eq!((dead.activity, dead.mobility), (0.0, 0.0), "{dead:?}");
    assert!(dead.scars > 0.9 && (0.3..=0.7).contains(&dead.lowland));
    assert!(hot.activity > 0.9 && hot.mobility < hot.activity, "{hot:?}");
    assert!(hot.scars < 0.05);
    let molten = PlanetRecord {
        temperature: 1600.0,
        ..PlanetRecord::TEMPERATE
    };
    let old = Interior::of(5, [EARTH, 9.81], &molten, 12.0);
    assert_eq!(old.activity, 1.0);
}

/// An old, cold, small world has dead plates and craters: nothing spreads or sinks, each of its ranges is
/// worn down to a quarter, so two that cross reach a half, its plates do not move, no volcano grows and
/// impact basins dig kilometres deep.
#[test]
fn old_cold_small_worlds_are_dead_and_scarred() {
    let plates = old_small();
    assert_eq!(span(&plates, |s| s.floor), (0.0, 0.0));
    assert_eq!(span(&plates, |s| s.cone), (0.0, 0.0));
    let (rift, range) = span(&plates, |s| s.rise);
    assert!(rift == 0.0 && range <= 0.75, "ranges reach {range}");
    let (pit, rim) = span(&plates, |s| s.pit);
    assert!(pit < -1500.0 && rim > 100.0, "basins from {pit} to {rim} m");
    assert_eq!(plates.motion(0, DVec3::X), DVec3::ZERO);
}

/// A young hot world has volcanoes and no basins, and a wet world as old as the Earth has ridges,
/// trenches, full ranges and plates that move along the ground.
#[test]
fn young_and_wet_worlds_are_active() {
    let hot = young_hot();
    assert!(span(&hot, |s| s.cone).1 > 0.1);
    assert_eq!(span(&hot, |s| s.pit), (0.0, 0.0));
    let home = earth(7);
    let (trench, ridge) = span(&home, |s| s.floor);
    assert!(
        trench < -0.1 && ridge > 0.1,
        "sea floor from {trench} to {ridge}"
    );
    assert!(span(&home, |s| s.rise).1 > 0.15);
    for dir in sphere_dirs(500) {
        let s = home.sample(dir);
        let motion = home.motion(s.plate, dir);
        assert!(motion.dot(dir).abs() < 1e-9 && motion.length() <= 1.0);
    }
    assert!(sphere_dirs(500).any(|d| home.motion(home.sample(d).plate, d).length() > 0.1));
}
