//! The stall: gentle, nose down, speed back, never a spin.

use bevy::prelude::*;

use super::{DT, Flight, gliding, looking};
use crate::gear::{GearInput, Mode};

/// Returns an input that looks 60 degrees up along the current flight path heading, so the
/// pilot pulls hard and asks for no turn.
fn pulling(velocity: Vec3) -> GearInput {
    let yaw = (-velocity.x).atan2(-velocity.z);
    GearInput {
        look: looking(yaw, 1.05),
        ..gliding(0.0)
    }
}

/// A hard pull-up held into the stall: the nose comes back down below the horizon and the
/// speed recovers on its own, while the pilot still pulls.
#[test]
fn stall_drops_the_nose_and_recovers_speed() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -25.0, -60.0));
    let (mut slowest, mut highest) = (f32::MAX, f32::MIN);
    for _ in 0..1800 {
        flight.tick(&pulling(flight.state.velocity), DT);
        slowest = slowest.min(flight.state.airspeed);
        highest = highest.max(flight.state.pitch);
    }
    assert!(highest > 0.3, "the nose never rose: {highest}");
    assert!(
        flight.state.pitch < 0.0,
        "the nose stayed up: {}",
        flight.state.pitch
    );
    assert!(
        flight.state.airspeed > slowest + 3.0,
        "speed did not recover"
    );
    assert!(
        flight.state.aoa < 0.33,
        "still stalled at {}",
        flight.state.aoa
    );
}

/// A stall entered in a bank levels the wings one way only: the roll rate never changes sign
/// and the bank never crosses to the other side. No spin.
#[test]
fn stall_never_spins() {
    for bank in [0.6, -0.6, 0.15] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
        flight.state.bank = bank;
        for frame in 0..1200 {
            flight.tick(&pulling(flight.state.velocity), DT);
            let rate = flight.state.servo.rates.y;
            assert!(
                rate * bank <= 1e-4,
                "frame {frame}: roll rate reversed to {rate}"
            );
            assert!(
                flight.state.bank * bank >= -1e-4,
                "frame {frame}: bank crossed over"
            );
        }
        assert!(
            flight.state.bank.abs() < 0.02,
            "wings not level after the stall"
        );
    }
}

/// A level stall with centred sticks stays exactly symmetric: no roll at all.
#[test]
fn level_stall_stays_level() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
    for _ in 0..1200 {
        flight.tick(&gliding(1.05), DT);
        assert!(flight.state.bank.abs() < 1e-4 && flight.state.velocity.x.abs() < 1e-3);
    }
}
