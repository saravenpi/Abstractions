//! Tests of the units: each boundary of distance, and the shared unit of a list.

use super::{distance, distance_short, distances, duration, gravity, mass, pressure, speed};
use crate::universe::{AU, LIGHT_SPEED, LIGHT_YEAR};

/// Every boundary of distance writes the unit and the number a person would say.
#[test]
fn distances_read_plainly_at_each_boundary() {
    let table = [
        (0.0, "0 m"),
        (340.0, "340 m"),
        (999.0, "999 m"),
        (1_000.0, "1 km"),
        (1_250.0, "1.25 km"),
        (12_400_000.0, "12 400 km"),
        (9_999_000.0, "10 000 km"),
        (3.2e9, "3.2 million km"),
        (1e10, "0.07 AU"),
        (0.1 * AU, "0.1 AU"),
        (1.5 * AU, "1.5 AU"),
        (1_000.0 * AU, "1 000 AU"),
        (0.5 * LIGHT_YEAR, "0.5 light-year"),
        (4.2 * LIGHT_YEAR, "4.2 light-years"),
        (100_000.0 * LIGHT_YEAR, "100 000 light-years"),
        (2.5e6 * LIGHT_YEAR, "2.5 million light-years"),
        (3e9 * LIGHT_YEAR, "3 billion light-years"),
    ];
    for (metres, words) in table {
        assert_eq!(distance(metres), words, "{metres}");
    }
    assert_eq!(distance(-5.0), "0 m");
    assert_eq!(distance_short(4.2 * LIGHT_YEAR), "4.2 ly");
}

/// The rows of one list share one unit, the one the largest needs.
#[test]
fn a_list_shares_one_unit() {
    let shown = distances(&[0.5 * LIGHT_YEAR, 4.2 * LIGHT_YEAR, 12.0 * LIGHT_YEAR]);
    assert_eq!(
        shown,
        ["0.5 light-year", "4.2 light-years", "12 light-years"].map(String::from)
    );
    let near = distances(&[300.0, 5_000.0, 80_000.0]);
    assert_eq!(near, ["0.3 km", "5 km", "80 km"].map(String::from));
}

/// Speeds, times, masses, gravity and air carry their unit.
#[test]
fn quantities_carry_their_unit() {
    assert_eq!(speed(12.0), "12 m/s");
    assert_eq!(speed(7_500.0), "7.5 km/s");
    assert_eq!(speed(1_200.0 * LIGHT_SPEED), "1 200 x light speed");
    let times = [
        (45.0, "45 s"),
        (180.0, "3 min"),
        (7_800.0, "2 h 10 min"),
        (432_000.0, "5 days"),
    ];
    for (seconds, words) in times {
        assert_eq!(duration(seconds), words);
    }
    assert_eq!(
        (mass(0.3), mass(12.0), mass(1_500.0)),
        ("300 g".into(), "12 kg".into(), "1.5 t".into())
    );
    assert_eq!(gravity(11.8), "1.2 x home, 11.8 m/s2");
    assert_eq!(pressure(0.6), "0.6 x home, 0.6 bar");
}
