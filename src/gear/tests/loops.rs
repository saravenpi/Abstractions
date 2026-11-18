//! Loops: a held pitch key takes the wings over the top and all the way round.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::glide::sticks;
use super::{DT, Flight, flying, gliding};
use crate::gear::aim::wrap;
use crate::gear::tune::CLIMB_HOLD;
use crate::gear::{GearInput, Mode};

/// Holds the back key on a powered `flight` until its nose has come nearly all the way round,
/// and returns the highest it got, its slowest speed and whether it flew backward over the top.
fn over_the_top(flight: &mut Flight) -> (f64, f32, bool) {
    let held = GearInput {
        pitch: 1.0,
        ..flying(0.0)
    };
    let (mut top, mut slowest, mut backward) = (0.0_f64, f32::MAX, false);
    for _ in 0..3600 {
        if flight.state.pitch >= TAU - 0.3 {
            break;
        }
        flight.tick(&held, DT);
        top = top.max(flight.pos.y);
        slowest = slowest.min(flight.state.airspeed);
        backward |= flight.state.velocity.z > 5.0 && (flight.state.pitch - PI).abs() < 0.6;
    }
    (top, slowest, backward)
}

/// From fast level flight, the back key held flies a whole loop: the body climbs, passes over
/// the top on its back flying the other way, comes down and, let go, levels out on the
/// heading it started on, without a jolt and without ever standing still in the air.
#[test]
fn pulling_up_flies_a_whole_loop() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    flight.run(&flying(0.0), 1.0);
    let (top, slowest, backward) = over_the_top(&mut flight);
    let pitch = flight.state.pitch;
    assert!(
        pitch >= TAU - 0.3 && backward,
        "pitch {pitch}, over: {backward}"
    );
    assert!(
        top > 15.0 && slowest > 8.0,
        "{top} m high, {slowest} m/s at the slowest"
    );
    flight.run(&flying(0.0), 3.0);
    let state = flight.state;
    let level = wrap(state.pitch);
    assert!(level.abs() < CLIMB_HOLD + 0.1, "pitch {level}");
    assert!(state.velocity.z < -30.0 && state.servo.yaw.abs() < 0.2);
    assert!(flight.twist < 0.2, "the body snapped by {}", flight.twist);
}

/// From a plain glide, far too slow for a round loop, a held pitch key still takes the body
/// all the way round, up and over or down and under: the wing pulls while it can, then the
/// body turns on through the stall and falls through. It never hangs with the nose tied to a
/// falling path, never spins and never snaps. Without the key the nose stays on the path.
#[test]
fn a_held_stick_turns_a_slow_glide_all_the_way_round() {
    let mut glide = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    glide.run(&gliding(0.0), 20.0);
    let turned = |stick: f32, sign: f32| {
        let (mut flight, start) = (glide, glide.state.pitch);
        let mut round = 0.0;
        for _ in 0..1200 {
            if round >= TAU {
                break;
            }
            flight.tick(&sticks(stick, 0.0), DT);
            round = sign * (flight.state.pitch - start);
        }
        assert!(flight.twist < 0.2, "{stick}: snapped by {}", flight.twist);
        round
    };
    assert!(
        glide.state.airspeed < 45.0,
        "a glide at {}",
        glide.state.airspeed
    );
    assert!(turned(1.0, 1.0) >= TAU && turned(-1.0, -1.0) >= TAU);
    assert!(turned(0.0, 1.0) < 2.0, "the nose left the path by itself");
}
