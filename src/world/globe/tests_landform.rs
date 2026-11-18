//! Tests of the landforms: dunes, craters and glaciers each shape the ground of their own class only.

use bevy::math::DVec3;

use super::Globe;
use super::craters::{DEPTH, FOOT, profile};
use super::landform::Landform;
use super::tests_record::sphere_dirs;
use crate::universe::{PlanetClass, PlanetRecord, Water};
use crate::world::{SEA_LEVEL, Surface};

/// Returns a 6 km globe of seed 42 at home gravity with a record of `class`, an `ocean` share and a mean
/// `temperature`, its water in the state the class has.
pub(super) fn world(class: PlanetClass, ocean: f64, temperature: f64) -> Globe {
    let water = match class {
        PlanetClass::Ocean | PlanetClass::Island => Water::Liquid,
        PlanetClass::Ice => Water::Ice,
        PlanetClass::Lava => Water::Steam,
        _ => Water::None,
    };
    let pressure = [1.0, 0.0][usize::from(class == PlanetClass::Barren)];
    let record = PlanetRecord {
        class,
        ocean,
        temperature,
        water,
        pressure,
        ..PlanetRecord::TEMPERATE
    };
    Globe::new(42, 6000.0, 22.0).with_record(record)
}

/// Returns the height `globe` adds over a dry world of islands of the same seed at each of `n` sites.
fn added(globe: &Globe, n: u32) -> Vec<(DVec3, f32)> {
    let plain = world(PlanetClass::Island, 0.0, 288.0);
    sphere_dirs(n)
        .map(|d| (d, globe.height(d) - plain.height(d)))
        .collect()
}

/// Deserts have dunes, worlds without air craters, ice worlds glaciers, and every other class nothing.
#[test]
fn each_class_has_its_landform() {
    for class in PlanetClass::ALL {
        let expected = match class {
            PlanetClass::Desert => Landform::Dunes,
            PlanetClass::Barren => Landform::Craters,
            PlanetClass::Ice => Landform::Glaciers,
            _ => Landform::Plain,
        };
        assert_eq!(Landform::of(class), expected, "{class:?}");
    }
}

/// Dunes only raise the ground, by at most 12 m, over much of a desert, and their crests run north to
/// south across the east and west winds: the ground rises and falls faster going east than going north.
#[test]
fn dunes_cross_the_wind_on_deserts() {
    let desert = world(PlanetClass::Desert, 0.0, 320.0);
    let dunes = added(&desert, 6000);
    assert!(dunes.iter().all(|(_, h)| (0.0..=12.0).contains(h)));
    let risen = dunes.iter().filter(|(_, h)| *h > 1.0).count();
    assert!(
        (1200..5400).contains(&risen),
        "{risen} of 6000 sites have dunes"
    );
    let (r, plain) = (desert.radius(), world(PlanetClass::Island, 0.0, 288.0));
    let dune = |d: DVec3| desert.height(d) - plain.height(d);
    let swing = |d: DVec3, along: DVec3| (dune((d * r + along * 6.0).normalize()) - dune(d)).abs();
    let equator = dunes.iter().filter(|(d, h)| d.y.abs() < 0.15 && *h > 1.0);
    let (mut east, mut north) = (0.0, 0.0);
    for (d, _) in equator {
        east += swing(*d, DVec3::Y.cross(*d).normalize());
        north += swing(*d, DVec3::Y);
    }
    assert!(east > 2.0 * north, "east {east} north {north}");
}

/// Craters dig bowls and raise rims over a world without air: a floor a fifth of the width under the rim
/// crest, nothing past 1.6 radii, and pits and rims over a good share of the ground.
#[test]
fn craters_pit_worlds_without_air() {
    let size = 100.0;
    assert!((profile(1.0, size) - profile(0.0, size) - DEPTH * size).abs() < 1e-4);
    assert_eq!(profile(FOOT, size), 0.0);
    assert!((profile(0.9999, size) - profile(1.0, size)).abs() < 0.01);
    let barren = world(PlanetClass::Barren, 0.0, 250.0);
    let craters = added(&barren, 6000);
    let pits = craters.iter().filter(|(_, h)| *h < -2.0).count();
    let rims = craters.iter().filter(|(_, h)| *h > 0.5).count();
    let touched = craters.iter().filter(|(_, h)| h.abs() > 0.01).count();
    assert!(pits > 60 && rims > 60, "{pits} pits and {rims} rims");
    assert!(
        (300..4000).contains(&touched),
        "{touched} of 6000 sites in craters"
    );
}

/// A cold ice world carves glacier valleys into its land and freezes its old sea flat at the datum, a
/// walkable snow sheet; a warm one, with no cold anywhere, only freezes the sea.
#[test]
fn glaciers_fill_cold_valleys_and_the_sea_freezes() {
    let plain = world(PlanetClass::Island, 0.0, 288.0);
    let (cold, warm) = (
        world(PlanetClass::Ice, 0.0, 200.0),
        world(PlanetClass::Ice, 0.0, 330.0),
    );
    let mut valleys = 0;
    for dir in sphere_dirs(6000) {
        let (base, frozen) = (plain.height(dir), cold.height(dir));
        assert_eq!(warm.height(dir), base.max(SEA_LEVEL), "{dir}");
        assert!(frozen >= SEA_LEVEL && frozen <= base.max(SEA_LEVEL));
        valleys += usize::from(frozen < base - 3.0);
        if frozen == SEA_LEVEL {
            assert_eq!(cold.surface(dir), Surface::Snow, "{dir}");
        }
    }
    assert!(valleys > 40, "{valleys} sites in glacier valleys");
}
