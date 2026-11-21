//! The view that steers: it turns the body the short way round, and the right way up however
//! the body and the view are read after a half loop.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::loops::back;
use super::pilot::powered;
use super::sense::posed;
use super::{DT, Flight};
use crate::gear::aim::wrap;
use crate::gear::{GearInput, Mode};

/// Returns the heading of the flight path of `flight`.
fn heading(flight: &Flight) -> f32 {
    let v = flight.state.velocity;
    (-v.x).atan2(-v.z)
}

/// A view that steers 179 degrees to the left of the flight path banks and turns the body to the
/// left, and one 181 degrees to the left, which is 179 to the right, to the right: the short
/// way round both times, from every heading, across the half turn where the angles wrap too.
#[test]
fn a_view_almost_behind_steers_the_short_way() {
    for step in 0..12 {
        let from = wrap(step as f32 * TAU / 12.0);
        for (aside, side) in [(179.0_f32, 1.0), (181.0, -1.0)] {
            let mut flight = posed(from, (-0.2, 0.0));
            let input = GearInput {
                deploy: true,
                aim: Some(Vec2::new(from + aside.to_radians(), -0.2)),
                ..GearInput::default()
            };
            flight.run(&input, 1.0);
            let (bank, turn) = (flight.state.bank, wrap(heading(&flight) - from));
            assert!(bank * side < -0.2, "{from}, {aside}: banked {bank}");
            assert!(turn * side > 0.05, "{from}, {aside}: turned {turn}");
        }
    }
}

/// Returns a powered flight that flew a half loop and let go on the top, the right way up
/// again and flying back the way it came.
fn reversed() -> Flight {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    flight.run(&powered(0.0, 0.0), 2.0);
    let mut frames = 0;
    while back(&flight) > -0.8 && frames < 600 {
        flight.tick(&powered(1.0, 0.0), DT);
        frames += 1;
    }
    flight.run(&powered(0.0, 0.0), 3.0);
    flight
}

/// After a half loop the gear reads the body past the vertical, its heading a half turn from
/// where it flies. A view that then steers a little to the right of the flight path, read the
/// upright way as the camera reads it once it has righted itself, keeps the back of the body to
/// the sky, puts the right wing down and turns the path to the right. Read on its back, as the
/// camera holds it over the top of the loop, the same view does the same.
///
/// Until this test the gear compared the two headings as they were read: it swung the nose a
/// half turn round and rolled the body onto its back.
#[test]
fn a_view_steers_the_right_way_after_a_half_loop() {
    let start = reversed();
    assert!(start.state.pitch.cos() < 0.0 && back(&start) > 0.7);
    let (from, rise) = (heading(&start), -0.1);
    let upright = Vec2::new(from - 0.5, rise);
    let on_its_back = Vec2::new(from - 0.5 + PI, PI - rise);
    for aim in [upright, on_its_back] {
        let mut flight = Flight {
            twist: 0.0,
            ..start
        };
        let input = GearInput {
            aim: Some(aim),
            ..powered(0.0, 0.0)
        };
        let mut wing = 0.0_f32;
        for _ in 0..180 {
            flight.tick(&input, DT);
            assert!(back(&flight) > 0.3, "{aim}: on its back");
            wing = wing.min((flight.state.frame() * Vec3::X).y);
        }
        let turn = wrap(heading(&flight) - from);
        assert!(
            wing < -0.2,
            "{aim}: the right wing went no lower than {wing}"
        );
        assert!(turn < -0.1 && turn > -0.7, "{aim}: turned {turn}");
        assert!(flight.twist < 0.12, "{aim}: snapped by {}", flight.twist);
    }
}
