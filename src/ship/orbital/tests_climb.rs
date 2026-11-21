//! Tests of the held height of a manual cruise: the vertical stick moves it at a few percent of
//! the height a second, at most 2 km/s, easing in and out, and stops at the edge of the band.

use super::rig::{DT, band, earth, skim_at};
use super::skim::{Ask, step};
use super::tune::{CLIMB, EASE};

/// Holding the stick up for 5 s at 200 km raises the held height by 5 to 10 km: the rate eases
/// in over a second to 2 km/s, never past it, and never changes by more than the easing allows
/// in a frame.
#[test]
fn holding_the_stick_moves_the_height_smoothly() {
    let planet = earth();
    let band = band(&planet);
    let mut skim = skim_at(&planet, 200_000.0);
    let ask = Ask {
        throttle: 1.0,
        climb: 1.0,
        ..Ask::default()
    };
    let (start, mut last) = (skim.height, 0.0);
    for _ in 0..(5.0 / DT) as usize {
        let stride = step(&skim, &band, &ask, DT);
        let rate = (stride.skim.height - skim.height) / DT;
        assert!(rate <= CLIMB.2 + 1e-6, "{rate} m/s");
        assert!(
            (rate - last).abs() <= CLIMB.2 / EASE * DT + 1e-6,
            "{last} to {rate} m/s"
        );
        (skim, last) = (stride.skim, rate);
    }
    let gain = skim.height - start;
    assert!((5_000.0..10_000.0).contains(&gain), "{gain} m in 5 s");
}

/// Held up, the height stops at the edge of the lock sphere, and comes to rest there.
#[test]
fn the_held_height_stops_at_the_edge_of_the_band() {
    let planet = earth();
    let band = band(&planet);
    let mut skim = skim_at(&planet, 1_000_000.0);
    let ask = Ask {
        throttle: 1.0,
        climb: 1.0,
        ..Ask::default()
    };
    for _ in 0..(300.0 / DT) as usize {
        skim = step(&skim, &band, &ask, DT).skim;
        assert!(skim.height <= band.ceiling + 1e-6, "{} m", skim.height);
    }
    assert_eq!((skim.height, skim.rise), (band.ceiling, 0.0));
}
