//! The roll, pitch and yaw keys: a roll into a bank and a held pull turn steadily, let go the
//! wings level; a held yaw key turns flat.

use bevy::prelude::*;

use super::pilot::keys;
use super::{DT, Flight, gliding};
use crate::gear::aim::wrap;
use crate::gear::tune::BANK_LIMIT;
use crate::gear::{Gear, GearInput, Mode};

/// Slowest turn rate in radians per second a bank and a held pull give a settled glide: most
/// of [`crate::gear::tune::TURN_CAP`], the drift a wingsuit turns at.
const RATE: f32 = 0.5;

/// Seconds the roll key is held to bank into a turn: about 50 degrees.
const ROLL_IN: f32 = 0.35;

/// Seconds the wings take to come within [`LEVEL`] of level once the keys are let go.
const LEVEL_TIME: f32 = 1.3;

/// Bank in radians that counts as level: 3 degrees.
const LEVEL: f32 = 0.052;

/// Most the bank may pass level on its way back, in radians: half a degree, which nobody sees.
const PAST_LEVEL: f32 = 0.009;

/// Returns a hands-off glide, settled.
fn settled() -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&keys(0.0, 0.0), 30.0);
    flight
}

/// Returns the heading of the flight path of `flight`.
fn heading(flight: &Flight) -> f32 {
    let v = flight.state.velocity;
    (-v.x).atan2(-v.z)
}

/// A roll to the left and the back key held turn left at a steady rate, the bank held where the
/// roll took it once it stopped and the nose on the path: a bank and a pull, as a pilot turns.
#[test]
fn a_bank_and_a_pull_turn_steadily() {
    let mut flight = settled();
    flight.run(&keys(0.0, -1.0), ROLL_IN);
    flight.run(&keys(1.0, 0.0), 0.5);
    let bank = flight.state.bank;
    assert!(bank < -0.6 && bank > -1.4, "bank {bank}");
    let mut turned = 0.0;
    let mut last = heading(&flight);
    for _ in 0..90 {
        flight.tick(&keys(1.0, 0.0), DT);
        turned += wrap(heading(&flight) - last);
        last = heading(&flight);
        let aoa = flight.state.aoa;
        assert!(aoa.abs() < 0.35, "slipping at {aoa}");
        assert!(
            (flight.state.bank - bank).abs() < 0.1,
            "the bank left {bank}"
        );
    }
    let rate = turned / 1.5;
    assert!(rate > RATE, "turns at {rate} rad/s");
}

/// Let go, the wings come within 3 degrees of level within [`LEVEL_TIME`], one way only, and the
/// glide flies on straight where it was going.
#[test]
fn letting_go_levels_the_wings() {
    for roll in [-1.0, 1.0] {
        let mut flight = settled();
        flight.run(&keys(0.0, roll), ROLL_IN);
        let bank = flight.state.bank;
        for _ in 0..(LEVEL_TIME / DT) as u32 {
            flight.tick(&keys(0.0, 0.0), DT);
            let now = flight.state.bank;
            assert!(
                now * bank.signum() >= -PAST_LEVEL,
                "the bank crossed to {now}"
            );
        }
        let level = flight.state.bank;
        assert!(level.abs() < LEVEL, "bank {level}");
        flight.run(&keys(0.0, 0.0), 1.0);
        let from = heading(&flight);
        flight.run(&keys(0.0, 0.0), 3.0);
        let drift = wrap(heading(&flight) - from);
        assert!(drift.abs() < 0.1, "still turning by {drift}");
    }
}

/// A held yaw key turns the flight path flat at about the yaw rate, the wings level.
#[test]
fn a_held_yaw_key_turns_flat() {
    for yaw in [-1.0, 1.0] {
        let mut flight = settled();
        let from = heading(&flight);
        for _ in 0..180 {
            flight.tick(
                &GearInput {
                    yaw,
                    ..keys(0.0, 0.0)
                },
                DT,
            );
            assert!(
                flight.state.bank.abs() < 0.02,
                "banked {}",
                flight.state.bank
            );
        }
        let turned = wrap(heading(&flight) - from);
        assert!(
            turned * yaw < -1.0 && turned * yaw > -1.5,
            "turned {turned}"
        );
        assert!(wrap(flight.state.servo.yaw - heading(&flight)).abs() < 0.1);
    }
}

/// A view to one side, as it is while the steer button is held, banks the wings that way and
/// turns the flight path onto it, without swinging past it.
#[test]
fn an_aim_to_one_side_banks_and_turns_onto_it() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    let input = GearInput {
        aim: Some(Vec2::new(-1.2, -0.35)),
        ..gliding(0.0)
    };
    flight.run(&input, 0.5);
    assert!(
        flight.state.bank > 0.2,
        "an aim to the right must bank right"
    );
    let mut widest = 0.0_f32;
    for _ in 0..900 {
        flight.tick(&input, DT);
        widest = widest.max(flight.state.velocity.with_y(0.0).normalize().x);
        assert!(
            flight.state.bank < BANK_LIMIT + 0.01,
            "banked past the limit"
        );
    }
    let (heading, bank) = (heading(&flight), flight.state.bank);
    assert!((heading + 1.2).abs() < 0.1, "heading {heading}");
    assert!(bank.abs() < 0.05, "wings not level: {bank}");
    assert!(widest <= 1.2_f32.sin() + 0.03, "swung past the aim");
}

/// Flies a glide in a cross wind for `frames` frames, the roll key at `roll` for the first
/// [`ROLL_IN`] and then the back key held, and returns the largest bank on the way. With the
/// roll key at zero nothing is held.
fn in_wind(gear: &mut Gear, roll: f32, frames: u32) -> f32 {
    let mut most = 0.0_f32;
    for frame in 0..frames {
        let rolling = (frame as f32) < ROLL_IN / DT;
        let pull = f32::from(u8::from(roll != 0.0 && !rolling));
        let input = GearInput {
            wind: Vec3::new(9.0, 0.0, 0.0),
            ..keys(pull, roll * f32::from(u8::from(rolling)))
        };
        gear.drive(gear.state.velocity, input, DT);
        most = most.max(gear.state.bank.abs());
    }
    most
}

/// In a cross wind a bank and a pull turn as fast one way as the other: the wings fly the path
/// through the air, not the track over the ground.
#[test]
fn a_cross_wind_does_not_bias_a_held_turn() {
    let turned = |roll: f32| {
        let mut gear = Gear {
            state: settled().state,
            ..Gear::default()
        };
        in_wind(&mut gear, 0.0, 600);
        let from = gear.state.servo.yaw;
        let bank = in_wind(&mut gear, roll, 90);
        (wrap(gear.state.servo.yaw - from).abs(), bank)
    };
    let (left, right) = (turned(-1.0), turned(1.0));
    assert!(left.0 > 0.4 && right.0 > 0.4, "{left:?} and {right:?}");
    assert!((left.0 - right.0).abs() < 0.25, "{left:?} and {right:?}");
    assert!((left.1 - right.1).abs() < 0.08, "{left:?} and {right:?}");
}
