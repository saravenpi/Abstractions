//! Rolls: a held roll key rolls the body on round, the keys work in the frame of the body, and
//! let go the body rights itself.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use super::loops::back;
use super::pilot::{keys, powered};
use super::{DT, Flight};
use crate::gear::aim::wrap;
use crate::gear::roll::settle;
use crate::gear::{FlightState, GearInput, Mode};

/// Seconds a body let go upside down may take to face the sky again.
const RIGHTING: f32 = 1.5;

/// How far the back must face the sky for the body to count as the right way up.
const UPRIGHT: f32 = 0.9;

/// Returns a hands-off glide, settled.
fn settled() -> Flight {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -7.0, -30.0));
    flight.run(&keys(0.0, 0.0), 10.0);
    flight
}

/// Rolls `flight` with the roll key at `roll` until it rolled by `angle`, and returns the
/// seconds it took and how far the back faced the sky at the lowest.
fn rolled(flight: &mut Flight, roll: f32, angle: f32) -> (f32, f32) {
    let (mut turned, mut lowest, mut frames) = (0.0_f32, 1.0_f32, 0);
    while turned.abs() < angle && frames < 600 {
        let was = flight.state.bank;
        flight.tick(&keys(0.0, roll), DT);
        turned += wrap(flight.state.bank - was);
        lowest = lowest.min(back(flight));
        frames += 1;
    }
    (frames as f32 * DT, lowest)
}

/// Holds `input` on `flight` and returns the seconds until its back faces the sky.
fn righted(flight: &mut Flight, input: &GearInput) -> f32 {
    let mut frames = 0;
    while back(flight) < UPRIGHT && frames < 600 {
        flight.tick(input, DT);
        frames += 1;
    }
    frames as f32 * DT
}

/// A held roll key rolls a glide all the way round both ways, through flying on its back, in a
/// few seconds and without a snap.
#[test]
fn a_held_roll_key_rolls_all_the_way_round() {
    for roll in [-1.0, 1.0] {
        let mut flight = settled();
        let (seconds, lowest) = rolled(&mut flight, roll, TAU);
        assert!(seconds < 3.0, "{roll}: took {seconds} s");
        assert!(lowest < -0.8, "{roll}: never on its back: {lowest}");
        assert!(flight.twist < 0.09, "{roll}: snapped by {}", flight.twist);
    }
}

/// Let go upside down, after a half roll or at the top of a loop, the body comes the right way
/// up within [`RIGHTING`], and the wings fly on.
#[test]
fn let_go_upside_down_the_body_rights_itself() {
    let mut glide = settled();
    rolled(&mut glide, 1.0, PI);
    assert!(back(&glide) < -0.8, "not upside down: {}", back(&glide));
    let seconds = righted(&mut glide, &keys(0.0, 0.0));
    assert!(seconds < RIGHTING, "a half roll took {seconds} s");
    let mut looped = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -75.0));
    looped.run(&powered(0.0, 0.0), 2.0);
    let mut frames = 0;
    while back(&looped) > -0.8 && frames < 600 {
        looped.tick(&powered(1.0, 0.0), DT);
        frames += 1;
    }
    let seconds = righted(&mut looped, &powered(0.0, 0.0));
    assert!(seconds < RIGHTING, "the top of a loop took {seconds} s");
    for flight in [glide, looped] {
        assert!(
            flight.state.mode.winged() && flight.twist < 0.09,
            "{:?}",
            flight.state
        );
    }
}

/// Upside down the back key still pulls toward the back of the body, now toward the ground, as
/// a real wing does: the path bends down.
#[test]
fn upside_down_the_back_key_pulls_toward_the_back() {
    let mut flight = settled();
    rolled(&mut flight, 1.0, PI);
    let (before, dorsal) = (flight.state.velocity, flight.state.frame() * Vec3::Y);
    flight.run(&keys(1.0, 0.0), 0.3);
    let bent = flight.state.velocity - before;
    assert!(dorsal.y < -0.8, "not upside down: {dorsal}");
    assert!(
        bent.dot(dorsal) > 3.0,
        "bent {bent} against the back {dorsal}"
    );
}

/// Off the wings and away from a fall, a body read past the vertical is read upright again:
/// the same frame, the pitch back on the near side of the vertical.
#[test]
fn reading_the_body_upright_keeps_its_frame() {
    let mut state = FlightState {
        mode: Mode::BootFall,
        pitch: 2.5,
        bank: 0.4,
        ..FlightState::default()
    };
    state.servo.yaw = 0.3;
    let frame = state.frame();
    settle(&mut state);
    assert!(state.pitch.abs() <= PI / 2.0, "pitch {}", state.pitch);
    assert!(state.frame().angle_between(frame) < 1e-4, "{state:?}");
}
