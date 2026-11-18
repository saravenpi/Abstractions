//! Loops: the wings follow a view that goes over the top and all the way round.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::{DT, Flight, flying, gliding};
use crate::gear::{GearInput, Mode};

/// Pitch rate of the view in radians per second, as the back key gives it.
const PULL: f32 = 1.5;

/// Farthest the view leads the nose, as the controller holds it.
const LEASH: f32 = 0.9;

/// Returns the input of powered flight with the view pitched `pitch` radians round.
fn viewing(pitch: f32) -> GearInput {
    GearInput {
        aim: Some(Vec2::new(0.0, pitch)),
        ..flying(0.0)
    }
}

/// From fast level flight, pulling the view up and round flies a whole loop: the body climbs,
/// passes over the top on its back flying the other way, comes down and levels out on the
/// heading it started on, without a jolt and without ever standing still in the air.
#[test]
fn pulling_up_flies_a_whole_loop() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    flight.run(&flying(0.0), 1.0);
    let (mut view, mut top, mut slowest) = (0.0_f32, 0.0_f64, f32::MAX);
    let mut backward = false;
    for _ in 0..3600 {
        if view >= TAU {
            break;
        }
        view = (view + PULL * DT).min(flight.state.pitch + LEASH);
        flight.tick(&viewing(view), DT);
        top = top.max(flight.pos.y);
        slowest = slowest.min(flight.state.airspeed);
        backward |= flight.state.velocity.z > 5.0 && (flight.state.pitch - PI).abs() < 0.6;
    }
    assert!(
        view >= TAU && backward,
        "view {view}, over the top: {backward}"
    );
    assert!(
        top > 15.0 && slowest > 8.0,
        "{top} m high, {slowest} m/s at the slowest"
    );
    flight.run(&viewing(TAU), 3.0);
    let state = flight.state;
    assert!((state.pitch - TAU).abs() < 0.3, "pitch {}", state.pitch);
    assert!(state.velocity.z < -30.0 && state.servo.yaw.abs() < 0.2);
    assert!(flight.twist < 0.2, "the body snapped by {}", flight.twist);
}

/// From a plain glide, far too slow for a round loop, a held pitch stick still takes the body
/// all the way round, up and over or down and under: the wing pulls while it can, then the
/// body turns on through the stall and falls through. It never hangs with the nose tied to a
/// falling path, never spins and never snaps. Without the stick the same view leaves the nose
/// on the path, as before.
#[test]
fn a_held_stick_turns_a_slow_glide_all_the_way_round() {
    let mut glide = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    glide.run(&gliding(-0.2), 20.0);
    let turned = |stick: f32, sign: f32| {
        let (mut flight, start) = (glide, glide.state.pitch);
        let (mut view, mut round) = (start, 0.0);
        for _ in 0..1200 {
            if round >= TAU {
                break;
            }
            let led = sign * (view - start) + PULL * DT;
            view = start + sign * led.min(round + LEASH);
            let input = GearInput {
                aim: Some(Vec2::new(0.0, view)),
                pitch: stick,
                ..gliding(0.0)
            };
            flight.tick(&input, DT);
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
