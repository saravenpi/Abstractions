//! Volatile tests: ices reach the inner planets thinly, red dwarfs dry their close planets, and
//! cold liquids pool only where they are liquid.

use super::volatiles::{AMMONIA, FROST, HYDROCARBON, LAKES, carbon, lakes, reach, water};
use super::*;

/// A planet beyond the frost line gets all the ices, the Earth about two thirds, and a planet
/// close to its star a small share. Carbon follows the ices.
#[test]
fn ices_thin_inside_the_frost_line() {
    assert_eq!(reach(FROST), 1.0);
    assert_eq!(reach(0.001), 1.0);
    assert!((0.66..0.69).contains(&reach(1.0)), "{}", reach(1.0));
    assert!(reach(100.0) < 0.3 && reach(100.0) > reach(1000.0));
    assert_eq!(carbon(1.0, 0.01), 1.0);
    assert!((carbon(1.0, 1.0) - reach(1.0).sqrt()).abs() < 1e-12);
    assert_eq!(carbon(0.0, 1.0), 0.0);
}

/// A star like the Sun leaves an Earth its water. A red dwarf dries the planets in its
/// habitable zone in proportion to its mass, and no planet beyond the frost line.
#[test]
fn red_dwarfs_dry_their_close_planets() {
    assert_eq!(water(0.5, 1.0, 1.0), 0.5 * reach(1.0));
    assert_eq!(water(0.5, 1.0, 0.35), 0.5 * reach(1.0));
    assert!(water(0.5, 1.0, 0.1) < 0.6 * water(0.5, 1.0, 1.0));
    assert!(water(0.5, 1.0, 0.1) > water(0.5, 1.0, 0.08));
    assert!((water(0.5, 0.1, 0.1) - 0.5).abs() < 1e-12);
    assert_eq!(water(0.5, 0.1, 1.0), 0.5);
    let wet =
        |suns: f64| (0..1000).filter(move |i| water(f64::from(*i) / 1000.0, 0.5, suns) >= 0.1);
    assert!(wet(0.1).count() < wet(1.0).count());
}

/// Lakes of ammonia and of hydrocarbons form only between their freezing and boiling points,
/// under enough pressure, on the share of worlds that hold them, over 2 to 30 percent of the
/// ground. Thicker air lets a liquid stay to a higher temperature.
#[test]
fn cold_liquids_pool_where_they_are_liquid() {
    assert_eq!(lakes(&AMMONIA, 220.0, 1.0, 0.0), LAKES.0);
    assert!(lakes(&AMMONIA, 220.0, 1.0, 0.2499) > 0.29);
    assert_eq!(lakes(&AMMONIA, 220.0, 1.0, 0.25), 0.0);
    assert_eq!(lakes(&AMMONIA, 190.0, 1.0, 0.1), 0.0);
    assert_eq!(lakes(&AMMONIA, 245.0, 1.0, 0.1), 0.0);
    assert!(lakes(&AMMONIA, 245.0, 3.0, 0.1) > 0.0);
    assert_eq!(lakes(&AMMONIA, 220.0, 0.05, 0.1), 0.0);
    assert!(lakes(&HYDROCARBON, 94.0, 1.5, 0.1) > 0.0);
    assert_eq!(lakes(&HYDROCARBON, 94.0, 0.05, 0.1), 0.0);
    assert_eq!(lakes(&HYDROCARBON, 200.0, 1.0, 0.1), 0.0);
    for roll in (0..100).map(|i| f64::from(i) / 100.0) {
        let share = lakes(&HYDROCARBON, 100.0, 1.0, roll);
        assert!(share == 0.0 || (LAKES.0..=LAKES.1).contains(&share));
    }
}

/// A record lists the share of its surface under each liquid: the sea of a wet world is water,
/// the pools of a lava world are molten rock, and a dry world has none.
#[test]
fn records_list_their_liquids() {
    let home = PlanetRecord::TEMPERATE;
    assert_eq!(
        home.liquids(),
        [PlanetRecord::BALANCED_OCEAN, 0.0, 0.0, 0.0]
    );
    let lava = PlanetRecord {
        class: PlanetClass::Lava,
        water: Water::None,
        ocean: 0.2,
        ..home
    };
    assert_eq!(lava.liquids(), [0.0, 0.0, 0.0, 0.2]);
    let titan = PlanetRecord {
        class: PlanetClass::Ice,
        water: Water::Ice,
        ocean: 0.0,
        hydrocarbon: 0.05,
        ..home
    };
    assert_eq!(titan.liquids(), [0.0, 0.0, 0.05, 0.0]);
}
