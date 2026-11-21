//! Tests of the air the condensation trails read: a contrail forms only in the cold air high up
//! behind a burning engine, and a hull climbing through Mach 1 just under the clouds wraps itself
//! in its vapour cone as a diving one does.

use bevy::prelude::*;

use super::Flown;
use super::sky::column;
use crate::speed::{cone, contrail, saturation};
use crate::universe::{EARTH_RADIUS, PlanetRecord, STANDARD_GRAVITY, TOP_SCALES, heights};

/// The gravity and the radius of an Earth.
const EARTH: (f64, f64) = (STANDARD_GRAVITY, EARTH_RADIUS);

/// Returns the thickness of the temperate air `height` metres up, against its sea level.
fn thickness(height: f64) -> f32 {
    let [top, ..] = heights(&PlanetRecord::TEMPERATE, EARTH.0, EARTH.1);
    (-height / (top / TOP_SCALES)).exp() as f32
}

/// Returns how strongly a full engine leaves a contrail `height` metres up in temperate air.
fn trail_at(height: f64) -> f32 {
    let air = column(&PlanetRecord::TEMPERATE, EARTH, height);
    contrail(air.cold, air.pressure, thickness(height), 1.0)
}

/// Over a temperate Earth a full engine leaves no contrail near the ground, where the air is
/// too warm, and one high up, from about 6 km, where the air is colder than the
/// Schmidt-Appleman threshold; an idle engine leaves none, and above the air there is none.
#[test]
fn contrails_form_only_in_the_cold_air_high_up() {
    for metres in [0.0, 1000.0, 2000.0, 4000.0] {
        assert_eq!(trail_at(metres), 0.0, "{metres} m");
    }
    for metres in [8000.0, 10_000.0, 12_000.0] {
        assert!(trail_at(metres) > 0.9, "{metres} m: {}", trail_at(metres));
    }
    let high = column(&PlanetRecord::TEMPERATE, EARTH, 10_000.0);
    assert_eq!(
        contrail(high.cold, high.pressure, thickness(10_000.0), 0.0),
        0.0
    );
    let space = column(&PlanetRecord::TEMPERATE, EARTH, 2.0e5);
    assert_eq!(space.pressure, 0.0);
}

/// A hull climbing straight up through the speed of sound a hundred metres under the base of a
/// covering cloud layer shows its cone, as one diving at that speed does: the cone reads the
/// speed, not the way.
#[test]
fn the_cone_fires_on_the_way_up_at_mach_one() {
    let [_, base, _] = heights(&PlanetRecord::TEMPERATE, EARTH.0, EARTH.1);
    let height = base - 100.0;
    let air = column(&PlanetRecord::TEMPERATE, EARTH, height);
    assert!(air.below > 50.0 && air.below < 150.0, "{}", air.below);
    let wet = saturation(air.below, 0.9);
    let mut strengths = Vec::new();
    for way in [Vec3::Y, Vec3::NEG_Y] {
        let mut flown = Flown::default();
        let vel = way * air.sound;
        flown.advance(Some(vel), (Vec3::Y, 9.81), 0.1);
        let (speed, _) = flown.advance(Some(vel), (Vec3::Y, 9.81), 0.1);
        strengths.push(cone(speed / air.sound, thickness(height), wet));
    }
    assert!(strengths[0] > 0.5, "climbing: {}", strengths[0]);
    assert_eq!(strengths[0], strengths[1]);
}
