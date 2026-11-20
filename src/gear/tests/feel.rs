//! How the wings answer the keys: a level pull bends the path at once, a banked pull turns
//! the heading no faster than a wingsuit drifts round, and a folded body falls belly down.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::pilot::keys;
use super::{DT, Flight};
use crate::gear::aim::wrap;
use crate::gear::tune::TURN_CAP;
use crate::gear::{GearInput, Mode};

/// Fewest degrees a held back key bends a level flight path in its first half second.
const BEND: f32 = 10.0;

/// Steepest path in radians on which the turn rate is held to the cap: steeper, the path has
/// little heading left to turn (50 degrees).
const SHALLOW: f32 = 0.87;

/// Returns the heading of the flight path of `flight` and its angle above the horizon.
fn track(flight: &Flight) -> Vec2 {
    let v = flight.state.velocity;
    Vec2::new((-v.x).atan2(-v.z), v.y.atan2(v.with_y(0.0).length()))
}

/// Returns a hands-off glide at `speed`: settled at the glide speed, or just launched faster.
fn at(speed: f32) -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -0.2 * speed, -speed));
    flight.run(&keys(0.0, 0.0), if speed < 40.0 { 20.0 } else { 0.5 });
    flight
}

/// Returns the degrees a held back key bends the path of a glide at `speed` in half a second.
fn bent(speed: f32) -> f32 {
    let mut flight = at(speed);
    let from = track(&flight).y;
    flight.run(&keys(1.0, 0.0), 0.5);
    (track(&flight).y - from).to_degrees()
}

/// Returns the fastest and the mean heading rate in radians per second of a pull held for
/// `seconds` on a glide at `speed` rolled onto its side, while the path is shallow.
fn banked_turn(speed: f32, seconds: f32) -> (f32, f32) {
    let mut flight = at(speed);
    while flight.state.bank < FRAC_PI_2 - 0.05 {
        flight.tick(&keys(0.0, 1.0), DT);
    }
    let (mut fastest, mut turned, mut frames) = (0.0_f32, 0.0_f32, 0);
    for _ in 0..(seconds / DT) as u32 {
        let was = track(&flight).x;
        flight.tick(&keys(1.0, 0.0), DT);
        if track(&flight).y.abs() < SHALLOW {
            let rate = wrap(track(&flight).x - was).abs() / DT;
            (fastest, turned, frames) = (fastest.max(rate), turned + rate, frames + 1);
        }
    }
    (fastest, turned / frames.max(1) as f32)
}

/// A held back key bends a level path by [`BEND`] degrees and more in half a second, at the
/// glide speed and faster.
#[test]
fn a_level_pull_bends_the_path_at_once() {
    for speed in [26.0, 45.0] {
        let bend = bent(speed);
        assert!(bend > BEND, "{speed} m/s: bent {bend} degrees");
    }
}

/// Rolled onto its side, a held pull turns the heading no faster than [`TURN_CAP`] at the
/// glide speed, and slower at speed, yet still turns.
#[test]
fn a_banked_pull_turns_no_faster_than_the_cap() {
    for speed in [26.0, 55.0] {
        let (fastest, mean) = banked_turn(speed, 3.0);
        assert!(
            fastest < TURN_CAP * 1.15,
            "{speed} m/s: turns at {fastest} rad/s"
        );
        assert!(mean > 0.3, "{speed} m/s: turns only {mean} rad/s");
    }
}

/// Folded, a body falls belly to the air at about 82 m/s, slower than the same body diving
/// head first, which is held upright.
#[test]
fn a_belly_down_fall_is_slower_than_a_head_down_dive() {
    let fall = |flat: bool| {
        let mut flight = Flight::new(Mode::Fall, Vec3::new(0.0, -10.0, -30.0));
        for _ in 0..2400 {
            if !flat {
                (flight.state.prone, flight.state.servo.rates.z) = (0.0, 0.0);
            }
            flight.tick(&GearInput::default(), DT);
        }
        -flight.state.velocity.y
    };
    let (belly, head) = (fall(true), fall(false));
    assert!((75.0..90.0).contains(&belly), "belly down at {belly} m/s");
    assert!(head > belly + 20.0, "head down at {head} m/s");
}

/// Prints the level pull, the banked turn, a pull-up from 70 m/s and the falls. Run with
/// `--nocapture`.
#[test]
fn print_feel() {
    for speed in [26.0, 45.0, 62.0] {
        println!(
            "level pull at {speed}: {:5.1} degrees in 0.5 s",
            bent(speed)
        );
    }
    for speed in [26.0, 55.0] {
        let (fastest, mean) = banked_turn(speed, 3.0);
        println!(
            "banked pull at {speed}: fastest {:5.1} mean {:5.1} degrees per second",
            fastest.to_degrees(),
            mean.to_degrees()
        );
    }
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -70.0));
    while flight.state.velocity.length() > 30.0 && flight.state.servo.clock < 10.0 {
        flight.tick(&keys(1.0, 0.0), DT);
    }
    println!("pull-up from 70 to 30 m/s climbs {:5.1} m", flight.pos.y);
}
