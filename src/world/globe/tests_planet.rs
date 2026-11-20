//! Tests of the ground of a planet of real size: the land share its record asks for, on wet, dry and ice
//! worlds, and mountains and oceans as high and as deep as its pull allows.

use super::tests_record::sphere_dirs;
use super::{Globe, Grain};
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::world::SEA_LEVEL;

/// Returns a planet 6,000 km in radius under 9.81 m/s², 4.5 billion years old, of seed 7, with the home
/// record changed into `class` with an `ocean` share and `water`.
pub(super) fn planet(class: PlanetClass, ocean: f64, water: Water) -> Globe {
    let record = PlanetRecord {
        class,
        ocean,
        water,
        ..PlanetRecord::TEMPERATE
    };
    Globe::new(7, 6.0e6, 9.81).with_record(record).aged(4.5)
}

/// Returns the ground heights of `globe` on 20,000 sites, as a reader too coarse for rivers sees them,
/// lowest first.
pub(super) fn heights(globe: &Globe) -> Vec<f32> {
    let mut all: Vec<f32> = sphere_dirs(20_000)
        .map(|d| globe.height_seen(d, Grain::Far))
        .collect();
    all.sort_by(f32::total_cmp);
    all
}

/// Returns the share of `heights` at or under the sea level.
fn low_share(heights: &[f32]) -> f64 {
    heights.iter().filter(|&&h| h <= SEA_LEVEL).count() as f64 / heights.len() as f64
}

/// Returns the share of low ground the interior of `globe` gives it.
fn lowland(globe: &Globe) -> f64 {
    let high = globe
        .relief
        .high()
        .expect("a planet of real size has plates");
    high.plates().interior().lowland
}

/// On a planet 6,000 km in radius the sea covers the share its record gives it within 3 percent, from a
/// dry-leaning world to an ocean world, over 20,000 sites.
#[test]
fn the_sea_of_a_large_planet_covers_its_share() {
    for (class, ocean) in [
        (PlanetClass::Island, 0.3),
        (PlanetClass::Island, PlanetRecord::BALANCED_OCEAN),
        (PlanetClass::Ocean, 0.9),
    ] {
        let globe = planet(class, ocean, Water::Liquid);
        let got = low_share(&heights(&globe));
        assert!((got - ocean).abs() < 0.03, "asked {ocean}, got {got}");
    }
}

/// A dry planet and an ice planet of real size have no sea, and their low ground is the share their seed
/// gives them: dry basins on the first, and on the second the frozen sea, flat at the datum.
#[test]
fn dry_and_ice_planets_keep_their_low_ground() {
    let dry = planet(PlanetClass::Desert, 0.0, Water::None);
    let ice = planet(PlanetClass::Ice, 0.0, Water::Ice);
    assert!(!dry.has_sea() && !ice.has_sea());
    for (name, globe) in [("dry", &dry), ("ice", &ice)] {
        let (all, want) = (heights(globe), lowland(globe));
        assert!((0.3..=0.7).contains(&want));
        let got = low_share(&all);
        assert!((got - want).abs() < 0.03, "{name}: asked {want}, got {got}");
    }
    let frozen = heights(&ice);
    assert_eq!(frozen[0], SEA_LEVEL);
    assert!(heights(&dry)[0] < -1500.0);
}

/// A planet like the Earth has summits of kilometres, under what its rock can carry plus the regional
/// band, and an ocean floor 3 to 6 km down under most of its sea; a planet three times as heavy with the
/// same share of its heat left, so the same plates, has lower summits.
#[test]
fn mountains_and_oceans_have_real_heights() {
    let all = heights(&planet(PlanetClass::Island, 0.6, Water::Liquid));
    let (deepest, floor, top) = (all[0], all[all.len() / 4], all[all.len() - 1]);
    println!("deepest {deepest} m, sea floor {floor} m, summit {top} m");
    assert!((1500.0..12_000.0).contains(&top), "summit at {top} m");
    assert!(
        (-6500.0..-2500.0).contains(&floor),
        "sea floor at {floor} m"
    );
    assert!(
        (-13_000.0..-4000.0).contains(&deepest),
        "deepest at {deepest} m"
    );
    let heavy = Globe::new(7, 6.0e6, 29.4).aged(4.5);
    let light = Globe::new(7, 6.0e6, 9.81).aged(4.5 * 3f64.sqrt().recip());
    let (heavy, light) = (heights(&heavy), heights(&light));
    assert!(heavy[heavy.len() - 1] < light[light.len() - 1]);
}
