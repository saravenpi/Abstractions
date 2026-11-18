//! Loops: a held pitch key takes the wings over the top and all the way round.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::pilot::{Pilot, keys};
use super::{DT, Flight, flying, gliding};
use crate::gear::Mode;
use crate::gear::aim::wrap;
use crate::gear::tune::LEASH;

/// Holds the back key on a powered flight until its nose has come nearly all the way round,
/// and returns the highest it got, its slowest speed and whether it flew backward over the top.
fn over_the_top(pilot: &mut Pilot) -> (f64, f32, bool) {
    let (mut top, mut slowest, mut backward) = (0.0_f64, f32::MAX, false);
    for _ in 0..3600 {
        if pilot.flight.state.pitch >= TAU - 0.3 {
            break;
        }
        pilot.tick(&keys(1.0, 0.0), &flying(0.0), DT);
        let flight = &pilot.flight;
        top = top.max(flight.pos.y);
        slowest = slowest.min(flight.state.airspeed);
        backward |= flight.state.velocity.z > 5.0 && (flight.state.pitch - PI).abs() < 0.6;
    }
    (top, slowest, backward)
}

/// From fast level flight, the back key held flies a whole loop: the body climbs, passes over
/// the top on its back flying the other way and comes down. Let go, it flies on along the aim
/// the key left, on the heading it started on, without a jolt and without ever standing still
/// in the air.
#[test]
fn pulling_up_flies_a_whole_loop() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    flight.run(&flying(0.0), 1.0);
    let mut pilot = Pilot::on(flight, 0.0);
    let (top, slowest, backward) = over_the_top(&mut pilot);
    let pitch = pilot.flight.state.pitch;
    assert!(
        pitch >= TAU - 0.3 && backward,
        "pitch {pitch}, over: {backward}"
    );
    assert!(
        top > 15.0 && slowest > 8.0,
        "{top} m high, {slowest} m/s at the slowest"
    );
    let left = pilot.stick.aim;
    pilot.run(&keys(0.0, 0.0), &flying(0.0), 3.0);
    let (state, twist) = (pilot.flight.state, pilot.flight.twist);
    let off = wrap(pilot.stick.aim.y - state.pitch);
    assert_eq!(pilot.stick.aim, left, "the aim moved by itself");
    assert!(off.abs() < 0.3 && wrap(left.y).abs() < LEASH + 0.3, "{off}");
    assert!(state.velocity.z < -30.0 && state.servo.yaw.abs() < 0.2);
    assert!(twist < 0.2, "the body snapped by {twist}");
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
        let (mut pilot, start) = (Pilot::on(glide, glide.state.pitch), glide.state.pitch);
        let mut round = 0.0;
        for _ in 0..1200 {
            if round >= TAU {
                break;
            }
            pilot.tick(&keys(stick, 0.0), &gliding(0.0), DT);
            round = sign * (pilot.flight.state.pitch - start);
        }
        let twist = pilot.flight.twist;
        assert!(twist < 0.2, "{stick}: snapped by {twist}");
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
