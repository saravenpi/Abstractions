//! Tests of the gases of the air: what each kind of world is born with, what it loses and what
//! life makes of it.

use super::born::{frost, holds};
use super::tune::{HELIUM, HOME_ESCAPE, METHANE, OXYGEN};
use super::{Gas, Mix, born, escape};
use crate::universe::range_tests::sample;
use crate::universe::{PlanetClass, PlanetRecord, Water};

/// Returns the home record as another class of world at `temperature` kelvin under `pressure`
/// home atmospheres, with its water in `water`.
pub(super) fn world(
    class: PlanetClass,
    temperature: f64,
    pressure: f64,
    water: Water,
) -> PlanetRecord {
    PlanetRecord {
        class,
        temperature,
        pressure,
        water,
        ocean: [0.0, 0.6][usize::from(water == Water::Liquid)],
        ..PlanetRecord::TEMPERATE
    }
}

/// Returns the records and the escape speeds of an Earth without life, a Mars, a Venus under
/// the pressure cap and a Titan.
pub(super) fn four() -> [(PlanetRecord, f64); 4] {
    [
        (
            world(PlanetClass::Island, 288.0, 1.0, Water::Liquid),
            escape(1.0, 1.0),
        ),
        (
            world(PlanetClass::Desert, 210.0, 0.006, Water::None),
            escape(0.38, 0.53),
        ),
        (
            world(PlanetClass::Desert, 737.0, 5.0, Water::None),
            escape(0.9, 0.95),
        ),
        (
            world(PlanetClass::Ice, 94.0, 1.45, Water::Ice),
            escape(0.138, 0.404),
        ),
    ]
}

/// The air of the home planet is the Earth's: nitrogen first, 28.96 g/mol, an exponent of
/// 0.286, shares that add up to 1 and fall in order.
#[test]
fn home_air_is_the_air_of_the_earth() {
    let home = Mix::HOME;
    assert_eq!(home.main_gas(), Gas::Nitrogen);
    assert!((home.molar() - 0.028_96).abs() < 2e-5, "{}", home.molar());
    assert!((home.kappa() - 0.286).abs() < 1e-3, "{}", home.kappa());
    let shares: Vec<f64> = home.parts().map(|part| part.share).collect();
    assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!(shares.windows(2).all(|pair| pair[0] >= pair[1]));
    assert_eq!(home.share(Gas::Oxygen), OXYGEN);
    assert_eq!(home.share(Gas::Hydrogen), 0.0);
}

/// An Earth without life is nitrogen with carbon dioxide and no oxygen, Mars and Venus are
/// carbon dioxide, Titan is nitrogen with methane, for every seed.
#[test]
fn the_record_picks_the_gases() {
    let [earth, mars, venus, titan] = four();
    for seed in 0..200 {
        let air = |(record, speed): (PlanetRecord, f64)| born(&record, speed, 0.0, seed);
        let (earth, mars, venus, titan) = (air(earth), air(mars), air(venus), air(titan));
        assert_eq!(earth.main_gas(), Gas::Nitrogen);
        assert!(earth.share(Gas::CarbonDioxide) >= 0.01 && earth.share(Gas::Oxygen) == 0.0);
        assert_eq!(
            (mars.main_gas(), venus.main_gas()),
            (Gas::CarbonDioxide, Gas::CarbonDioxide)
        );
        assert!(mars.share(Gas::CarbonDioxide) > 0.9 && venus.share(Gas::CarbonDioxide) > 0.9);
        assert_eq!(titan.main_gas(), Gas::Nitrogen);
        let methane = titan.share(Gas::Methane);
        assert!(METHANE.0 <= methane && methane <= METHANE.1, "{methane}");
        assert_eq!(titan.share(Gas::CarbonDioxide), 0.0);
    }
}

/// A gas giant is hydrogen and helium, molten ground and a moon kneaded by tides breathe
/// sulphur dioxide, and boiled water stays in the air as steam.
#[test]
fn giants_and_hot_worlds_have_their_own_air() {
    let giant = world(PlanetClass::GasGiant, 165.0, 5.0, Water::None);
    let lava = world(PlanetClass::Lava, 1500.0, 2.0, Water::Steam);
    let moon = world(PlanetClass::Desert, 130.0, 0.02, Water::None);
    let steam = world(PlanetClass::Desert, 500.0, 3.0, Water::Steam);
    for seed in 0..200 {
        let jupiter = born(&giant, f64::INFINITY, 0.0, seed);
        assert_eq!(jupiter.main_gas(), Gas::Hydrogen);
        let helium = jupiter.share(Gas::Helium);
        assert!(HELIUM.0 <= helium && helium <= HELIUM.1, "{helium}");
        assert_eq!(
            born(&lava, escape(1.0, 1.0), 0.0, seed).main_gas(),
            Gas::SulphurDioxide
        );
        assert_eq!(
            born(&moon, escape(0.5, 0.5), 0.01, seed).main_gas(),
            Gas::SulphurDioxide
        );
        assert_eq!(
            born(&moon, escape(0.5, 0.5), 0.0, seed).main_gas(),
            Gas::Nitrogen
        );
        let wet = born(&steam, escape(1.0, 1.0), 0.0, seed);
        assert_eq!(wet.main_gas(), Gas::CarbonDioxide);
        assert!(wet.share(Gas::Steam) >= 0.1);
    }
}

/// The Earth holds nitrogen but not hydrogen, carbon dioxide frost forms at 195 K under one
/// atmosphere and 148 K under the air of Mars, and a body with almost no pull keeps a trace of
/// the heaviest gas of its recipe and nothing else.
#[test]
fn a_weak_pull_loses_the_light_gases() {
    assert!(holds(Gas::Nitrogen, 288.0, HOME_ESCAPE));
    assert!(!holds(Gas::Hydrogen, 288.0, HOME_ESCAPE));
    assert!(holds(Gas::Methane, 94.0, escape(0.138, 0.404)));
    assert!((frost(1.0) - 194.7).abs() < 1e-9 && (frost(0.006) - 148.0).abs() < 1.0);
    let [earth, mars, _, titan] = four();
    for (record, heaviest) in [
        (earth.0, Gas::CarbonDioxide),
        (mars.0, Gas::CarbonDioxide),
        (titan.0, Gas::Argon),
    ] {
        let trace = born(&record, 20.0, 0.0, 5);
        assert_eq!(trace.parts().count(), 1);
        assert_eq!((trace.main_gas(), trace.share(heaviest)), (heaviest, 1.0));
    }
}

/// Life that splits water turns the carbon dioxide into oxygen at the Earth's share and leaves
/// nitrogen first; doing it to the air of the Earth changes nothing to speak of.
#[test]
fn life_breathes_oxygen_into_the_air() {
    let (earth, speed) = four()[0];
    for seed in 0..200 {
        let alive = born(&earth, speed, 0.0, seed).breathed();
        assert_eq!(alive.main_gas(), Gas::Nitrogen);
        assert!((alive.share(Gas::Oxygen) - OXYGEN).abs() < 1e-12);
        assert_eq!(alive.share(Gas::CarbonDioxide), 0.0);
        assert!(
            (alive.molar() - Mix::HOME.molar()).abs() < 3e-4,
            "{}",
            alive.molar()
        );
    }
    let again = Mix::HOME.breathed();
    assert!((again.share(Gas::Nitrogen) - Mix::HOME.share(Gas::Nitrogen)).abs() < 1e-3);
    assert!((again.molar() - Mix::HOME.molar()).abs() < 2e-5);
}

/// Over the star sample every planet has an air whose shares add up to 1 and fall in order,
/// the same for the same seed; every giant is hydrogen, no solid planet keeps hydrogen as its
/// main gas and none has oxygen, which only life makes. The main gases are counted and printed.
#[test]
fn generated_air_holds_together() {
    let mut seen = [0usize; 9];
    for (u, star) in sample() {
        let planets = u.system(star.id).planets;
        assert_eq!(planets, u.system(star.id).planets);
        for p in planets.iter().filter(|p| p.id != u.home()) {
            let mix = p.record.mix;
            let shares: Vec<f64> = mix.parts().map(|part| part.share).collect();
            assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-9, "{mix:?}");
            assert!(shares.windows(2).all(|pair| pair[0] >= pair[1]), "{mix:?}");
            let giant = p.record.class == PlanetClass::GasGiant;
            assert_eq!(giant, mix.main_gas() == Gas::Hydrogen, "{p:?}");
            assert_eq!(mix.share(Gas::Oxygen), 0.0, "{p:?}");
            seen[mix.main_gas() as usize] += 1;
        }
    }
    for (gas, count) in Gas::ALL.iter().zip(seen) {
        println!("{gas:?}: {count}");
    }
    assert!(seen[Gas::Nitrogen as usize] > 0 && seen[Gas::CarbonDioxide as usize] > 0);
}
