//! Tests of the record a globe carries: dry worlds, ocean worlds and the unchanged home.

use std::f64::consts::PI;

use bevy::math::DVec3;

use super::Globe;
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::{SEA_LEVEL, Surface};

/// Returns `n` unit directions spread evenly over the sphere on a golden spiral.
pub(super) fn sphere_dirs(n: u32) -> impl Iterator<Item = DVec3> {
    let golden = PI * (3.0 - 5f64.sqrt());
    (0..n).map(move |i| {
        let y = 1.0 - 2.0 * (f64::from(i) + 0.5) / f64::from(n);
        let (ring, turn) = ((1.0 - y * y).sqrt(), golden * f64::from(i));
        DVec3::new(ring * turn.cos(), y, ring * turn.sin())
    })
}

/// Returns a 6 km globe of seed 7 with the home record changed into `class` and an `ocean` share.
pub(super) fn globe(class: PlanetClass, ocean: f64) -> Globe {
    let record = PlanetRecord {
        class,
        ocean,
        ..PlanetRecord::TEMPERATE
    };
    Globe::new(7, 6000.0, 9.8).with_record(record)
}

/// Returns true when `surface` lies under a liquid.
fn wet(surface: Surface) -> bool {
    matches!(
        surface,
        Surface::Water | Surface::Shallows | Surface::Reef | Surface::Pond
    )
}

/// A dry globe of any class has no water, shallows, reef or pond on 2,400 sites, and no pond basin.
#[test]
fn a_dry_globe_has_no_water_anywhere() {
    for class in PlanetClass::ALL {
        let dry = globe(class, 0.0);
        assert!(!dry.has_sea() && dry.sea_level() == f32::NEG_INFINITY);
        let mut low = 0;
        for dir in sphere_dirs(2400) {
            let surface = dry.surface(dir);
            assert!(!wet(surface), "{class:?} has {surface:?} at {dir}");
            assert!(!dry.pond_at(dir), "{class:?} has a pond at {dir}");
            low += usize::from(dry.height(dir) <= SEA_LEVEL);
        }
        assert!(low > 800, "the old sea bed of {class:?} is gone: {low}");
    }
}

/// the ocean globe keeps some land.
#[test]
fn an_ocean_globe_has_more_water() {
    let count = |ocean: f64| {
        let globe = globe(PlanetClass::Ocean, ocean);
        let sites = sphere_dirs(2400).filter(|&d| wet(globe.surface(d)));
        sites.count()
    };
    let (sparse, balanced, ocean) = (count(0.2), count(PlanetRecord::BALANCED_OCEAN), count(0.93));
    assert!(
        sparse < balanced && balanced < ocean,
        "{sparse} {balanced} {ocean}"
    );
    assert!(ocean < 2400 && ocean > 2400 * 8 / 10, "{ocean}");
}

/// A globe given the home record is the globe built without one: same heights, biomes, surfaces and ponds.
#[test]
fn the_home_globe_is_unchanged() {
    let plain = Globe::new(7, 6000.0, 9.8);
    let home = Globe::new(7, 6000.0, 9.8).with_record(PlanetRecord::TEMPERATE);
    assert_eq!(*plain.record(), PlanetRecord::TEMPERATE);
    assert!(plain.has_sea() && plain.sea_level() == SEA_LEVEL);
    for dir in sphere_dirs(2000) {
        assert_eq!(plain.height(dir).to_bits(), home.height(dir).to_bits());
        assert_eq!(plain.biome(dir), home.biome(dir));
        assert_eq!(plain.surface(dir), home.surface(dir));
        assert_eq!(plain.pond_at(dir), home.pond_at(dir));
    }
    for (x, z) in (0..400).map(|i| (i as f32 * 37.3 - 7000.0, i as f32 * -29.1 + 5000.0)) {
        assert_eq!(
            plain.height_flat(x, z).to_bits(),
            home.height_flat(x, z).to_bits()
        );
        assert_eq!(plain.surface_flat(x, z), home.surface_flat(x, z));
    }
}
