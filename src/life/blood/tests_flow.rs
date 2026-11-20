//! Tests of the pressure, the flow and the clotting of blood.

use super::*;

/// A taller body has more pressure, and a stronger pull gives more.
#[test]
fn pressure_grows_with_height_and_gravity() {
    assert!(pressure(9.81, 3.0, true) > pressure(9.81, 1.0, true));
    assert!(pressure(20.0, 1.0, true) > pressure(5.0, 1.0, true));
    assert!(pressure(9.81, 1.0, false) < pressure(9.81, 1.0, true));
}

/// A giraffe holds about 250 mmHg (33 kPa) at the heart; the model stays within a third of it.
#[test]
fn a_tall_body_reaches_giraffe_pressure() {
    let kpa = pressure(9.81, 3.0, true) / 1000.0;
    assert!((22.0..44.0).contains(&kpa), "{kpa}");
}

/// An artery jets a metre or more at home pressure and the speed follows Bernoulli.
#[test]
fn an_artery_jets_a_few_metres_per_second() {
    let v = speed(pressure(9.81, 1.3, true));
    assert!((4.0..8.0).contains(&v), "{v}");
}

/// The flow grows with the size of the cut and with the pressure, and a cut wider than the
/// artery gives no more than the artery.
#[test]
fn flow_grows_with_the_wound_and_the_pressure() {
    let m = 100.0;
    let p = pressure(9.81, 1.3, true);
    assert!(gush(0.01, m, p) > gush(0.002, m, p));
    assert!(gush(0.01, m, 2.0 * p) > gush(0.01, m, p));
    assert_eq!(gush(1.0, m, p), gush(vessel(m), m, p));
    assert!(seep(0.1, 0.02) > seep(0.05, 0.02));
    assert!(seep(0.1, 0.02) > seep(0.1, 0.01));
    assert!(gush(0.03, m, p) > 10.0 * seep(0.1, 0.02));
}

/// Clotting only lowers the flow with time, faster for a shallow wound than for a major one.
#[test]
fn wounds_clot() {
    assert_eq!(clot(0.0, false), 1.0);
    assert!(clot(60.0, false) < clot(30.0, false));
    assert!(clot(60.0, true) > clot(60.0, false));
    assert!(clot(1000.0, false) < 1e-4);
}

/// The pulse has a mean of one over a beat and stops between beats.
#[test]
fn the_pulse_keeps_the_mean() {
    let rate = heart_rate(100.0);
    let steps = 4000;
    let mean: f32 = (0..steps)
        .map(|i| pulse(i as f32 / steps as f32 / rate, rate))
        .sum::<f32>()
        / steps as f32;
    assert!((mean - 1.0).abs() < 0.01, "{mean}");
    assert_eq!(pulse(0.75 / rate, rate), 0.0);
}

/// A bigger body holds more blood and beats slower.
#[test]
fn mass_sets_volume_and_heart() {
    assert!(volume(100.0) > 10.0 * volume(5.0));
    assert!((volume(70.0) - 4.6e-3).abs() < 0.5e-3);
    assert!(heart_rate(1.0) > heart_rate(1000.0));
}

/// A drop of a millilitre is a sphere of 6 mm that leaves a stain of 3.6 cm.
#[test]
fn a_millilitre_drop_is_six_millimetres() {
    let r = drop_radius(1.0e-6);
    assert!((r - 0.0062).abs() < 0.0003);
    assert!((stain(1.0e-6) - 0.036).abs() < 0.002);
    assert!(drag(0.003, 1.0) > drag(0.02, 1.0));
    assert_eq!(drag(0.01, 0.0), 0.0);
}
