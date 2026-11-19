//! Tests of the odds of life on chosen worlds: a cold moon like Titan and a hot airless rock.

use super::*;

/// Returns a world under a Sun-like star with `pressure`, `temperature`, `flux` and liquid share `ocean`.
fn world(pressure: f32, temperature: f32, flux: f32, ocean: f32) -> World {
    World {
        gravity: 1.35,
        pressure,
        temperature,
        flux,
        radius: 2575.0,
        ocean,
        star: 5780.0,
    }
}

/// A Titan-like moon, 94 K under 1.5 atmospheres with methane and ethane lakes, can live through
/// its hydrocarbons, but rarely, and never past mats. Prints how many of 4,000 seeds live.
#[test]
fn titan_lives_rarely_and_low() {
    let titan = world(1.5, 94.0, 0.011, 0.0);
    assert_eq!(
        chem::solvent(&titan).map(|(s, _)| s),
        Some(Solvent::Hydrocarbon)
    );
    let rungs: Vec<_> = (0..4000).filter_map(|seed| rung(&titan, seed)).collect();
    println!(
        "titan: {} of 4000 live, suits {:?}",
        rungs.len(),
        suitability(&titan)
    );
    assert!(!rungs.is_empty() && rungs.len() < 400);
    assert!(rungs.iter().all(|r| *r == Rung::Mats));
}

/// A hot dry airless rock never lives.
#[test]
fn hot_airless_rock_never_lives() {
    let rock = world(0.0, 700.0, 4.0, 0.0);
    assert!((0..2000).all(|seed| rung(&rock, seed).is_none()));
}

/// Water life climbs faster than ammonia life, which climbs faster than hydrocarbon life.
#[test]
fn cold_chemistry_is_slow() {
    assert!(speed(288.0) == 1.0 && speed(400.0) == 1.0);
    assert!(speed(220.0) < 0.7 && speed(220.0) > speed(94.0) && speed(94.0) < 0.05);
}
