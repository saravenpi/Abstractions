//! Tests of the sea datum: the mapping from the ocean share to the sea height, and what it does to the ground.

use super::Globe;
use super::datum::datum;
use super::tests_record::{globe, sphere_dirs};
use crate::universe::{PlanetClass, PlanetRecord};
use crate::world::SEA_LEVEL;

/// Returns the share of `n` sites spread over `globe` whose ground lies at or below the sea level.
fn low_share(globe: &Globe, n: u32) -> f64 {
    let low = sphere_dirs(n).filter(|&d| globe.height(d) <= SEA_LEVEL);
    low.count() as f64 / f64::from(n)
}

/// The balanced share gives a datum of exactly 0, a larger share always a higher sea, and no share no sea.
#[test]
fn the_sea_datum_is_monotone_and_exact_when_balanced() {
    assert_eq!(datum(PlanetRecord::BALANCED_OCEAN), Some(0.0));
    assert_eq!(datum(0.0), None);
    assert_eq!(datum(-0.2), None);
    let levels: Vec<f32> = (1..=100)
        .filter_map(|i| datum(f64::from(i) / 100.0))
        .collect();
    assert_eq!(levels.len(), 100);
    assert!(levels.windows(2).all(|w| w[0] < w[1]), "{levels:?}");
    assert!(levels[9] < levels[29] && levels[29] < 0.0);
    assert!(levels[89] > levels[69] && levels[69] > 0.0);
    assert_eq!(datum(1.0), datum(7.0));
}

/// The sea covers about the share of the sphere its record asks for, on two seeds and two radii.
#[test]
fn the_sea_covers_the_share_it_is_given() {
    for (seed, radius) in [(7, 6000.0), (42, 3000.0), (1, 40_000.0)] {
        for share in [0.1, 0.3, 0.6, 0.8, 0.95] {
            let record = PlanetRecord {
                ocean: share,
                ..PlanetRecord::TEMPERATE
            };
            let globe = Globe::new(seed, radius, 9.8).with_record(record);
            let got = low_share(&globe, 3000);
            assert!(
                (got - share).abs() < 0.1,
                "seed {seed} radius {radius}: asked {share}, got {got}"
            );
        }
    }
}

/// Raising or lowering the sea moves every height by the datum and nothing else, away from the ponds.
#[test]
fn the_datum_is_a_constant_shift() {
    let home = Globe::new(7, 6000.0, 9.8);
    for share in [0.25, 0.9] {
        let moved = globe(PlanetClass::Island, share);
        let shift = datum(share).unwrap_or(0.0);
        for dir in sphere_dirs(1500).filter(|&d| !home.pond_at(d) && !moved.pond_at(d)) {
            let (was, is) = (home.height(dir), moved.height(dir));
            let off = (is - (was - shift)).abs();
            assert!(
                off < 1e-3,
                "height moved by {off} more than the datum at {dir}"
            );
        }
    }
}
