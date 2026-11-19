//! The stall: the protection holds the nose down, the speed comes back, never a spin.

use bevy::prelude::*;

use super::pilot::keys;
use super::{DT, Flight};
use crate::gear::Mode;
use crate::gear::tune::STALL_SPEED;

/// The back key held for ten seconds from a fast glide: the wing pulls up and stalls, which
/// a held key may do, and the nose falls below the horizon with the key still held. Let go,
/// the glide speed comes back, with no tumble.
#[test]
fn stall_drops_the_nose_and_recovers_speed() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -10.0, -45.0));
    flight.run(&keys(0.0, 0.0), 1.0);
    let (mut slowest, mut lowest) = (f32::MAX, f32::MAX);
    for frame in 0..600 {
        flight.tick(&keys(1.0, 0.0), DT);
        slowest = slowest.min(flight.state.airspeed);
        if frame > 120 {
            lowest = lowest.min(flight.state.pitch);
        }
    }
    assert!(lowest < 0.0, "the nose stayed up: {lowest}");
    assert!(slowest < STALL_SPEED.0, "never stalled: {slowest}");
    flight.run(&keys(0.0, 0.0), 6.0);
    assert!(
        flight.state.airspeed > slowest + 8.0,
        "speed did not recover: {} from {slowest}",
        flight.state.airspeed
    );
    assert!(
        flight.state.aoa < 0.33,
        "still stalled at {}",
        flight.state.aoa
    );
    assert!(flight.twist < 0.09, "tumbled by {}", flight.twist);
}

/// A slow glide with the back key held: under the stall speed the nose comes down below the
/// horizon by itself within a second and a half.
#[test]
fn under_the_stall_speed_the_nose_comes_down() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -11.0));
    flight.state.pitch = 0.3;
    flight.run(&keys(1.0, 0.0), 1.5);
    assert!(
        flight.state.pitch < 0.0,
        "the nose stayed at {}",
        flight.state.pitch
    );
    assert!(flight.twist < 0.09, "tumbled by {}", flight.twist);
}

/// A stall entered in a bank, the roll key centred, levels the wings one way only: the roll
/// rate never changes sign and the bank never crosses to the other side. No spin.
#[test]
fn stall_never_spins() {
    for bank in [0.6, -0.6, 0.15] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
        flight.state.bank = bank;
        for frame in 0..1200 {
            let stick = f32::from(u8::from(frame < 90));
            flight.tick(&keys(stick, 0.0), DT);
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

/// A level stall with the roll key centred stays exactly symmetric: no roll at all.
#[test]
fn level_stall_stays_level() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
    for frame in 0..1200 {
        let stick = f32::from(u8::from(frame < 90));
        flight.tick(&keys(stick, 0.0), DT);
        let state = flight.state;
        assert!(state.bank.abs() < 1e-4 && state.velocity.x.abs() < 1e-3);
    }
}
