//! The sideslip: the body points where it flies through the air. After any manoeuvre the slip
//! dies away with the keys let go, and only a held yaw key holds a small one.

use bevy::prelude::*;

use super::loops::back;
use super::pilot::{keys, powered};
use super::{DT, Flight, gliding};
use crate::gear::{FlightState, GearInput, Mode};

/// Largest sideslip in radians a body may keep one second after the keys are let go (3 degrees).
const SETTLED: f32 = 0.052;

/// Returns the sideslip of `state` in radians: the angle between its velocity through the air
/// and the plane of its nose and its back, positive with the air coming from the right.
fn slip(state: &FlightState) -> f32 {
    let local = state.frame().inverse() * state.velocity;
    (local.x / state.velocity.length().max(1e-3))
        .clamp(-1.0, 1.0)
        .asin()
}

/// Returns a settled hands-off glide.
fn glide() -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -7.0, -30.0));
    flight.run(&keys(0.0, 0.0), 4.0);
    flight
}

/// Flies `flight` for `seconds` with every key let go under `idle`, and returns the largest
/// sideslip over the last frame of it.
fn let_go(mut flight: Flight, idle: &GearInput, seconds: f32) -> f32 {
    flight.run(idle, seconds);
    slip(&flight.state).abs()
}

/// A banked turn flown by the keys, a steer of the view a full radian to the side, and a half
/// loop under power that leaves the body read past the vertical: one second after the keys and
/// the view let go, the body points within 3 degrees of where it flies through the air.
#[test]
fn the_slip_dies_away_after_every_manoeuvre() {
    let mut turned = glide();
    turned.run(&keys(0.0, 1.0), 0.5);
    turned.run(&keys(1.0, 0.0), 2.0);
    let mut steered = glide();
    let aside = GearInput {
        aim: Some(Vec2::new(1.0, -0.25)),
        ..gliding(-0.25)
    };
    steered.run(&aside, 3.0);
    let mut looped = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    looped.run(&powered(0.0, 0.0), 2.0);
    let mut frames = 0;
    while back(&looped) > -0.8 && frames < 600 {
        looped.tick(&powered(1.0, 0.0), DT);
        frames += 1;
    }
    for (what, flight, idle) in [
        ("turn", turned, keys(0.0, 0.0)),
        ("steer", steered, keys(0.0, 0.0)),
        ("half loop", looped, powered(0.0, 0.0)),
    ] {
        let left = let_go(flight, &idle, 1.0);
        assert!(
            left < SETTLED,
            "{what}: slips {} degrees",
            left.to_degrees()
        );
    }
}

/// The yaw key held turns the path flat, and the nose lags it by a small slip, under 10 degrees,
/// which it keeps while the key is held. Let go, the slip is gone within a second.
#[test]
fn a_held_yaw_key_holds_a_small_slip_that_ends_when_let_go() {
    let mut flight = glide();
    let rudder = GearInput {
        yaw: 1.0,
        ..keys(0.0, 0.0)
    };
    flight.run(&rudder, 1.5);
    let mut held = Vec::new();
    for _ in 0..30 {
        flight.tick(&rudder, DT);
        held.push(slip(&flight.state).abs());
    }
    let (least, most) = held
        .iter()
        .fold((f32::MAX, 0.0_f32), |(a, b), s| (a.min(*s), b.max(*s)));
    assert!(
        least > 0.005 && most < 0.17,
        "held slip {least} to {most} rad"
    );
    let left = let_go(flight, &keys(0.0, 0.0), 1.0);
    assert!(
        left < SETTLED * 0.5,
        "slips {} degrees once let go",
        left.to_degrees()
    );
}
