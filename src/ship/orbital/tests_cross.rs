//! Tests of the speed of the orbital cruise: half an Earth-size planet in under 100 s at full
//! lever, at one height above the air, and a cruise to the far side that stops where the glide
//! begins.

use std::f64::consts::PI;

use bevy::math::DVec3;

use super::glide::handover;
use super::rig::{DT, band, earth, skim_at};
use super::skim::{Ask, step};

/// Seconds the cruise may take to cross half a planet at full lever: 90 s at full speed, and the
/// spool of the drive.
const LIMIT: f64 = 100.0;

/// Seconds a cruise to a target on the far side may take: the same, and the slowing down to
/// where the glide begins.
const TO_TARGET: f64 = 110.0;

/// At full lever the cruise sweeps half an Earth-size planet in under [`LIMIT`] seconds, at the
/// height it started at, above the air, over 200 km/s.
#[test]
fn the_cruise_crosses_half_a_planet_in_under_a_hundred_seconds() {
    let planet = earth();
    let band = band(&planet);
    let mut skim = skim_at(&planet, band.floor);
    let ask = Ask {
        throttle: 1.0,
        ..Ask::default()
    };
    let (mut swept, mut time) = (0.0, 0.0);
    while swept < PI && time < 2.0 * LIMIT {
        let stride = step(&skim, &band, &ask, DT);
        swept += skim.pos.angle_between(stride.skim.pos);
        let height = stride.skim.pos.length() - planet.radius;
        assert!((height - band.floor).abs() < 1.0, "{height} m at {time} s");
        assert!(height > planet.atmosphere_height);
        skim = stride.skim;
        time += DT;
    }
    assert!(time < LIMIT, "half a planet took {time} s");
    assert!(skim.speed > 200_000.0, "only {} m/s", skim.speed);
}

/// A cruise to a target on the far side flies straight to it and stops where the glide
/// begins, never past it, in under [`TO_TARGET`] seconds.
#[test]
fn a_cruise_to_the_far_side_stops_where_the_glide_begins() {
    let planet = earth();
    let band = band(&planet);
    let goal = DVec3::NEG_X;
    let mut skim = skim_at(&planet, band.floor);
    let ask = Ask {
        throttle: 1.0,
        goal: Some(goal),
        ..Ask::default()
    };
    let (mut time, mut done) = (0.0, false);
    while !done && time < 2.0 * TO_TARGET {
        let stride = step(&skim, &band, &ask, DT);
        assert!(stride.skim.height >= band.floor - 1e-6);
        (skim, done, time) = (stride.skim, stride.done, time + DT);
    }
    assert!(done && time < TO_TARGET, "done {done} after {time} s");
    let arc = skim.pos.angle_between(goal) * planet.radius;
    let wanted = handover(band.floor);
    assert!(
        (arc - wanted).abs() < 2000.0,
        "stopped {arc} m out, not {wanted}"
    );
}
