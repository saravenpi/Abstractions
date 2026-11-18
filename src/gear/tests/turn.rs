//! The roll keys: a held key banks into a steady coordinated turn, let go the wings level.

use bevy::prelude::*;

use super::glide::sticks;
use super::{DT, Flight, gliding};
use crate::gear::Mode;
use crate::gear::aim::wrap;

/// Slowest and fastest turn rate in radians per second a held roll key gives a settled glide.
const RATE: (f32, f32) = (0.45, 0.8);

/// Most the sink rate may grow in a held turn, as a share of the straight glide.
const SINK_SHARE: f32 = 0.6;

/// Seconds the wings take to come level once the roll key is let go.
const LEVEL_TIME: f32 = 1.5;

/// Returns a glide settled hands off.
fn settled() -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(0.0), 30.0);
    flight
}

/// Returns the heading of the flight path of `flight`.
fn heading(flight: &Flight) -> f32 {
    let v = flight.state.velocity;
    (-v.x).atan2(-v.z)
}

/// A held left key banks left and turns left at a steady rate, the nose on the path, and the
/// glide sinks at most [`SINK_SHARE`] faster than straight.
#[test]
fn a_held_roll_key_turns_steadily() {
    let mut flight = settled();
    let straight = -flight.state.velocity.y;
    flight.run(&sticks(0.0, -1.0), 2.0);
    assert!(flight.state.bank < -0.5, "bank {}", flight.state.bank);
    let (from, mut turned, mut sink) = (heading(&flight), 0.0, 0.0_f32);
    let mut last = from;
    for _ in 0..240 {
        flight.tick(&sticks(0.0, -1.0), DT);
        turned += wrap(heading(&flight) - last);
        last = heading(&flight);
        sink = sink.max(-flight.state.velocity.y);
        assert!(
            flight.state.aoa.abs() < 0.3,
            "slipping at {}",
            flight.state.aoa
        );
    }
    let rate = turned / 4.0;
    assert!((RATE.0..=RATE.1).contains(&rate), "turns at {rate} rad/s");
    assert!(
        sink < straight * (1.0 + SINK_SHARE),
        "sinks {sink} against {straight}"
    );
}

/// Let go, the wings come level within [`LEVEL_TIME`], one way only, and the glide flies on
/// straight where it was going.
#[test]
fn letting_go_levels_the_wings() {
    for roll in [-1.0, 1.0] {
        let mut flight = settled();
        flight.run(&sticks(0.0, roll), 3.0);
        let bank = flight.state.bank;
        for _ in 0..(LEVEL_TIME / DT) as u32 {
            flight.tick(&gliding(0.0), DT);
            assert!(flight.state.bank * bank >= -1e-4, "the bank crossed over");
        }
        assert!(flight.state.bank.abs() < 0.05, "bank {}", flight.state.bank);
        let from = heading(&flight);
        flight.run(&gliding(0.0), 3.0);
        let drift = wrap(heading(&flight) - from);
        assert!(drift.abs() < 0.1, "still turning by {drift}");
    }
}
