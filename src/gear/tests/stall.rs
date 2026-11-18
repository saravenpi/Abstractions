//! The stall: gentle, nose down, speed back, never a spin.

use bevy::prelude::*;

use super::glide::sticks;
use super::{DT, Flight, gliding};
use crate::gear::Mode;

/// Airspeed in metres per second under which the wing is stalled.
const STALL: f32 = 15.0;

/// Pulls `flight` up with the back key until it is slower than [`STALL`], and returns the
/// frames it took. Fails if the speed never drops that far.
fn pull_to_stall(flight: &mut Flight) -> u32 {
    for frame in 0..600 {
        if flight.state.airspeed < STALL {
            return frame;
        }
        flight.tick(&sticks(1.0, 0.0), DT);
    }
    panic!("never slowed under {STALL}: {}", flight.state.airspeed);
}

/// A pull-up held into a stall and let go: the nose comes back down below the horizon, the
/// speed builds again and the angle of attack comes back inside the wing, with no tumble.
#[test]
fn stall_drops_the_nose_and_recovers_speed() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -10.0, -45.0));
    flight.run(&gliding(0.0), 1.0);
    pull_to_stall(&mut flight);
    let (mut slowest, mut lowest) = (f32::MAX, f32::MAX);
    for _ in 0..360 {
        flight.tick(&gliding(0.0), DT);
        slowest = slowest.min(flight.state.airspeed);
        lowest = lowest.min(flight.state.pitch);
    }
    assert!(lowest < 0.0, "the nose stayed up: {lowest}");
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

/// A stall entered in a bank levels the wings one way only: the roll rate never changes sign
/// and the bank never crosses to the other side. No spin.
#[test]
fn stall_never_spins() {
    for bank in [0.6, -0.6, 0.15] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
        flight.state.bank = bank;
        for frame in 0..1200 {
            let stick = f32::from(u8::from(frame < 90));
            flight.tick(&sticks(stick, 0.0), DT);
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

/// A level stall with the roll stick centred stays exactly symmetric: no roll at all.
#[test]
fn level_stall_stays_level() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -30.0));
    for frame in 0..1200 {
        let stick = f32::from(u8::from(frame < 90));
        flight.tick(&sticks(stick, 0.0), DT);
        assert!(flight.state.bank.abs() < 1e-4 && flight.state.velocity.x.abs() < 1e-3);
    }
}
