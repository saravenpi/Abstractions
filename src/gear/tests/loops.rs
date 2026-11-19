//! Loops: a held back key flies the wings over the top and round when they have the speed for
//! it, and stalls out of the climb when they do not.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::prelude::*;

use super::pilot::{keys, powered};
use super::{DT, Flight};
use crate::gear::aim::wrap;
use crate::gear::{GearInput, Mode};

/// Largest turn of the body in one frame of a sixtieth of a second, in radians: a slow body
/// turns fast over the top of a loop, a snap turns it in a single step and far more.
const SNAP: f32 = 0.12;

/// Speed in metres per second a glide needs to loop without the boosters.
const LOOP_SPEED: f32 = 70.0;

/// What a held key did to the body.
#[derive(Debug, Default)]
struct Held {
    /// How far the pitch turned, counted the whole way round.
    turned: f32,
    /// How far the back faced the sky with the body over the top, the lowest being upside down.
    top: f32,
    /// Lowest airspeed on the way.
    slowest: f32,
}

/// Returns how far the back of the body faces the sky.
pub(super) fn back(flight: &Flight) -> f32 {
    (flight.state.frame() * Vec3::Y).y
}

/// Holds `input` on `flight` until its pitch turned all the way round, or for `seconds`.
fn hold(flight: &mut Flight, input: &GearInput, seconds: f32) -> Held {
    let mut held = Held {
        top: 1.0,
        slowest: f32::MAX,
        ..Held::default()
    };
    for _ in 0..(seconds / DT) as u32 {
        let was = flight.state.pitch;
        flight.tick(input, DT);
        held.turned += wrap(flight.state.pitch - was);
        held.slowest = held.slowest.min(flight.state.airspeed);
        if (held.turned - PI).abs() < 0.3 {
            held.top = held.top.min(back(flight));
        }
        if held.turned >= TAU - 0.2 {
            break;
        }
    }
    held
}

/// From powered flight and from a glide fast enough for it, as after a long dive, the back key
/// held flies a whole loop: up past the vertical, over the top on its back, down and round. Let
/// go, the wings fly on the right way up, and the body never snapped.
#[test]
fn a_held_back_key_loops_at_flying_speed() {
    let mut fast = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    fast.run(&powered(0.0, 0.0), 2.0);
    let glide = Flight::new(Mode::Glide, Vec3::new(0.0, 0.0, -LOOP_SPEED));
    let flights = [(fast, powered(1.0, 0.0)), (glide, keys(1.0, 0.0))];
    for (mut flight, input) in flights {
        let speed = flight.state.velocity.length();
        let held = hold(&mut flight, &input, 14.0);
        assert!(held.turned >= TAU - 0.2, "from {speed} m/s: {held:?}");
        assert!(
            held.top < -0.8,
            "from {speed} m/s, not on its back: {held:?}"
        );
        let release = GearInput {
            pitch: 0.0,
            ..input
        };
        flight.run(&release, 3.0);
        let state = flight.state;
        assert!(back(&flight) > 0.9 && state.mode.winged(), "{state:?}");
        assert!(
            state.airspeed > 20.0,
            "from {speed} m/s, ends at {}",
            state.airspeed
        );
        assert!(
            flight.twist < SNAP,
            "from {speed} m/s, snapped by {}",
            flight.twist
        );
    }
}

/// From a plain glide or a slow one the back key held climbs, and the wing, too slow to turn the
/// path over the top, stalls: the nose never passes the vertical, the body never ends up on its
/// back, and it falls out nose down with the key still held.
#[test]
fn a_held_back_key_stalls_out_of_a_slow_climb() {
    for speed in [30.0, 20.0] {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -0.2 * speed, -speed));
        flight.run(&keys(0.0, 0.0), 1.0);
        let mut lowest = f32::MAX;
        let mut highest = f32::MIN;
        for frame in 0..360 {
            flight.tick(&keys(1.0, 0.0), DT);
            highest = highest.max(flight.state.pitch);
            if frame > 120 {
                lowest = lowest.min(flight.state.pitch);
            }
            assert!(
                back(&flight) > -0.3,
                "{speed} m/s, frame {frame}: on its back"
            );
        }
        assert!(highest < FRAC_PI_2, "{speed} m/s: looped to {highest}");
        assert!(lowest < 0.0, "{speed} m/s: the nose stayed up at {lowest}");
        assert!(
            flight.twist < SNAP,
            "{speed} m/s: snapped by {}",
            flight.twist
        );
    }
}
