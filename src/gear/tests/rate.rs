//! Frame rate independence: the same inputs end in the same place at 30, 60 and 144 Hz.

use bevy::prelude::*;

use super::{Flight, braking, flying, gliding};
use crate::gear::{GearInput, Mode};

/// Returns where a body ends after 10 s of `input` from `start` at `rate` frames per second.
fn end(start: &Flight, input: &GearInput, rate: u32) -> (Vec3, Vec3) {
    let mut flight = *start;
    for _ in 0..rate * 10 {
        flight.tick(input, 1.0 / rate as f32);
    }
    (flight.pos.as_vec3(), flight.state.velocity)
}

/// Checks that position and velocity at 30 and 144 Hz are within 2 percent of those at 60 Hz.
fn same_at_every_rate(start: Flight, input: GearInput) {
    let (pos, vel) = end(&start, &input, 60);
    for rate in [30, 144] {
        let (p, v) = end(&start, &input, rate);
        let (off, slip) = ((p - pos).length(), (v - vel).length());
        assert!(
            off < 0.02 * pos.length(),
            "{rate} Hz: {off} m off over {}",
            pos.length()
        );
        assert!(
            slip < 0.02 * vel.length(),
            "{rate} Hz: {slip} m/s off at {}",
            vel.length()
        );
    }
}

/// A climbing powered turn from a jump.
#[test]
fn powered_turn_is_rate_independent() {
    let input = GearInput {
        roll: 0.3,
        aim: Some(Vec2::new(1.0, 0.2)),
        ..flying(0.0)
    };
    same_at_every_rate(Flight::new(Mode::Off, Vec3::new(0.0, 7.0, -5.0)), input);
}

/// A diving, turning glide.
#[test]
fn glide_is_rate_independent() {
    let input = GearInput {
        pitch: -0.4,
        aim: Some(Vec2::new(-0.7, -0.5)),
        ..gliding(0.0)
    };
    same_at_every_rate(
        Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0)),
        input,
    );
}

/// A fall caught by the boots with a drift, and a plain folded fall.
#[test]
fn falls_are_rate_independent() {
    let drift = GearInput {
        pitch: -1.0,
        roll: 0.5,
        ..braking()
    };
    same_at_every_rate(Flight::new(Mode::Off, Vec3::new(4.0, -50.0, 0.0)), drift);
    same_at_every_rate(
        Flight::new(Mode::Fall, Vec3::new(20.0, 5.0, 0.0)),
        GearInput::default(),
    );
}
