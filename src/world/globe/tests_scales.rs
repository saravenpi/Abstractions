//! Tests of the heights a planet can carry: mountains by its gravity, oceans by their share, both by its
//! size, and no plates at all on a world the local ground shapes alone.

use bevy::math::DVec3;

use super::Globe;
use super::elevation::tune::{DEPTH, PEAK};
use super::relief::{Scales, weight};
use super::tests_planet::planet;
use crate::universe::{PlanetClass, Water};

/// The highest mountain is what the rock carries under the gravity, within 3 to 9 km; the ocean is 3 to
/// 6 km deep; a planet as small as the Moon gets lower ranges.
#[test]
fn heights_follow_the_gravity_and_the_size() {
    let earth = Scales::of(6.371e6, 9.81, 0.7);
    assert!(
        (earth.peak - 8797.0).abs() < 5.0 && earth.scale == 1.0,
        "{earth:?}"
    );
    assert!((earth.deep - 5100.0).abs() < 1.0);
    assert!((Scales::of(1.0e7, 20.0, 0.5).peak - 4315.0).abs() < 1.0);
    assert_eq!(Scales::of(1.0e7, 40.0, 0.0).peak, PEAK.0 as f32);
    assert_eq!(Scales::of(1.0e7, 3.0, 0.0).peak, PEAK.1 as f32);
    assert_eq!(Scales::of(1.0e7, 9.81, 0.0).deep, DEPTH.0 as f32);
    assert_eq!(Scales::of(1.0e7, 9.81, 1.0).deep, DEPTH.1 as f32);
    let moon = Scales::of(1.737e6, 1.62, 0.5);
    assert!(
        (0.2..0.35).contains(&moon.scale) && moon.peak < 3000.0,
        "{moon:?}"
    );
}

/// A world of 50 km or less has no plates at all, so the local ground shapes it alone as it always did; the
/// plates count in full from 400 km, and a planet without ground has none.
#[test]
fn small_worlds_have_no_plates() {
    for radius in [3000.0, 6000.0, 40_000.0, 49_000.0] {
        assert_eq!(weight(radius), 0.0);
        assert_eq!(Scales::of(radius, 22.0, 0.6).peak, 0.0);
        assert!(Globe::new(7, radius, 22.0).plate(DVec3::X).is_none());
    }
    assert_eq!(weight(400_000.0), 1.0);
    assert!(
        planet(PlanetClass::Island, 0.6, Water::Liquid)
            .plate(DVec3::X)
            .is_some()
    );
    assert!(
        planet(PlanetClass::GasGiant, 0.0, Water::None)
            .plate(DVec3::X)
            .is_none()
    );
}
