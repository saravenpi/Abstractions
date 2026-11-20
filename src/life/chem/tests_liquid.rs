//! Tests of the liquids life can use.

use super::*;

/// Returns a world under home air pressure `pressure` at `temperature` kelvin, with liquid share `ocean`.
fn world(pressure: f32, temperature: f32, ocean: f32) -> World {
    World {
        gravity: 22.0,
        pressure,
        temperature,
        flux: 1.0,
        radius: 6000.0,
        ocean,
        star: 5780.0,
        tilt: 0.0,
        liquids: [
            ocean.max(f32::MIN_POSITIVE),
            0.0375,
            0.0375,
            ocean.max(f32::MIN_POSITIVE),
        ],
    }
}

/// Water out of its liquid range scores 0, and pressure moves the boiling point up.
#[test]
fn water_outside_its_range_scores_zero() {
    assert_eq!(suitability(&world(1.0, 260.0, 0.6), Solvent::Water), 0.0);
    assert_eq!(suitability(&world(1.0, 380.0, 0.6), Solvent::Water), 0.0);
    assert!(suitability(&world(5.0, 380.0, 0.6), Solvent::Water) > 0.0);
}

/// Water life does best from 273 to 313 K, not at the middle of the liquid range, and seas count
/// more than damp ground.
#[test]
fn water_suits_best_in_its_band() {
    let at = |t, ocean| suitability(&world(1.0, t, ocean), Solvent::Water);
    assert_eq!(at(288.0, 0.6), 1.0);
    assert_eq!(at(310.0, 0.6), 1.0);
    assert!(at(350.0, 0.6) < at(320.0, 0.6) && at(320.0, 0.6) < 1.0);
    assert!(at(288.0, 0.0) < 0.5 * at(288.0, 0.6));
}

/// Liquids combine as independent chances, and a world with none scores 0.
#[test]
fn liquids_combine_as_chances() {
    let home = world(1.0, 288.0, 0.6);
    assert_eq!(wet(&home), 1.0);
    assert_eq!(wet(&world(1.0, 800.0, 0.0)), 0.0);
    let cold = world(1.0, 220.0, 0.0);
    assert_eq!(wet(&cold), suitability(&cold, Solvent::Ammonia));
}
