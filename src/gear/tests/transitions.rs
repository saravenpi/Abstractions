//! Changes mid-air: every one keeps the velocity continuous and turns the attitude gently.

use bevy::prelude::*;

use super::{DT, Flight, braking, flying, gliding};
use crate::gear::{GearInput, Mode};

/// Largest velocity change in one frame in the air: 6 g, the load limit plus thrust and weight.
pub(super) const AIR_KICK: f32 = 6.0 * 22.0 * DT;

/// Largest attitude change in one frame, in radians (about 5 degrees).
pub(super) const TWIST: f32 = 0.09;

/// Checks that no frame of `flight` so far jumped the velocity or the attitude.
pub(super) fn assert_smooth(flight: &Flight) {
    assert!(flight.kick < AIR_KICK, "velocity jumped by {}", flight.kick);
    assert!(flight.twist < TWIST, "attitude jumped by {}", flight.twist);
}

/// Flies `flight` through `stages` of input and seconds and checks every frame was smooth.
fn smooth_through(mut flight: Flight, stages: &[(GearInput, f32)]) -> Flight {
    for (input, seconds) in stages {
        flight.run(input, *seconds);
    }
    assert_smooth(&flight);
    flight
}

/// Spreading the wings in a fast fall and folding them in a fast glide.
#[test]
fn deploy_and_fold_mid_air() {
    let idle = GearInput::default();
    for fall in [-115.0, -40.0, -5.0] {
        let falling = Flight::new(Mode::Fall, Vec3::new(0.0, fall, -3.0));
        let end = smooth_through(falling, &[(idle, 1.0), (gliding(0.0), 8.0), (idle, 3.0)]);
        assert_eq!(end.state.mode, Mode::Fall);
        assert_eq!(end.state.wings, 0.0);
    }
}

/// Switching the boosters on and off in flight, and the wings on and off under boost.
#[test]
fn boosters_and_wings_switch_mid_flight() {
    let flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    let stages = [
        (gliding(0.0), 2.0),
        (flying(0.2), 4.0),
        (gliding(-0.3), 3.0),
        (flying(0.0), 3.0),
        (braking(), 4.0),
        (flying(0.1), 4.0),
    ];
    let end = smooth_through(flight, &stages);
    assert_eq!(end.state.mode, Mode::Fly);
}

/// Stopping flight mid-air: the wings fold in about 0.3 s, the velocity carries on
/// ballistically, and the boots then catch the fall.
#[test]
fn stopping_mid_air_is_ballistic_then_boots_catch() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
    flight.run(&flying(0.0), 2.0);
    let before = flight.state.velocity;
    flight.run(&GearInput::default(), 0.35);
    assert_eq!(flight.state.mode, Mode::Fall);
    assert_eq!(flight.state.wings, 0.0, "wings still open after 0.35 s");
    assert!(flight.state.velocity.with_y(0.0).length() > 0.8 * before.length());
    let folded = flight.state.velocity;
    flight.tick(&GearInput::default(), DT);
    let accel = (flight.state.velocity - folded) / DT;
    assert!((accel.y + 22.0).abs() < 1.0, "not ballistic: {accel}");
    assert!(
        accel.with_y(0.0).length() < 9.0,
        "air brake too hard: {accel}"
    );
    flight.run(&braking(), 6.0);
    assert!(
        flight.state.velocity.y > -5.5,
        "boots did not catch the fall"
    );
    assert_smooth(&flight);
}

/// Flying up out of the air on a climb set along the path: lift, drag and thrust fade and the
/// body goes ballistic, smoothly.
#[test]
fn running_out_of_air_is_smooth() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 20.0, -60.0));
    (flight.state.pitch, flight.state.servo.course) = (0.32, 0.32);
    for frame in 0..1200 {
        let density = 1.2 * (-(frame as f32) / 150.0).exp();
        let input = GearInput {
            density: if frame > 900 { 0.0 } else { density },
            ..flying(0.5)
        };
        flight.tick(&input, DT);
    }
    assert_smooth(&flight);
    let before = flight.state.velocity;
    flight.tick(&flying(0.5), DT);
    assert!(flight.state.velocity != before);
}

/// Entering water folds the gear at once and hands the velocity over untouched.
#[test]
fn water_folds_the_wings_and_hands_over() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(0.0), 1.0);
    let before = flight.state.velocity;
    let wet = GearInput {
        in_water: true,
        ..flying(0.0)
    };
    flight.tick(&wet, DT);
    assert_eq!(flight.state.mode, Mode::Off);
    assert_eq!(flight.state.velocity, before);
    flight.run(&wet, 0.4);
    assert_eq!(flight.state.wings, 0.0);
    assert_eq!(flight.state.velocity, before);
    assert!(flight.twist < TWIST, "attitude jumped by {}", flight.twist);
}
