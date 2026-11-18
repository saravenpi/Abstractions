//! The roll keys: a held key banks into a steady coordinated turn that carries the aim and
//! the view with it, let go the wings level.

use bevy::prelude::*;

use super::pilot::{Pilot, keys};
use super::{DT, Flight, gliding, looking};
use crate::gear::aim::wrap;
use crate::gear::stick::{Stick, steer};
use crate::gear::{Gear, GearInput, Mode};

/// Slowest and fastest turn rate in radians per second a held roll key gives a settled glide.
const RATE: (f32, f32) = (0.45, 0.8);

/// Most the sink rate may grow in a held turn, as a share of the straight glide.
const SINK_SHARE: f32 = 0.6;

/// Seconds the wings take to come level once the roll key is let go.
const LEVEL_TIME: f32 = 1.5;

/// Most the bank may pass level on its way back, in radians: half a degree, which nobody sees.
const PAST_LEVEL: f32 = 0.009;

/// The slope of the glide the test pilots aim along, in radians.
const SLOPE: f32 = -0.245;

/// Returns a pilot on a glide settled along [`SLOPE`].
fn settled() -> Pilot {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    flight.run(&gliding(SLOPE), 30.0);
    Pilot::on(flight, SLOPE)
}

/// Returns the heading of the flight path of `flight`.
fn heading(flight: &Flight) -> f32 {
    let v = flight.state.velocity;
    (-v.x).atan2(-v.z)
}

/// A held left key banks left and turns left at a steady rate, the nose on the path, and the
/// glide sinks at most [`SINK_SHARE`] faster than straight.
#[test]
fn a_held_roll_key_turns_steadily() {
    let mut pilot = settled();
    let straight = -pilot.flight.state.velocity.y;
    pilot.run(&keys(0.0, -1.0), &gliding(0.0), 2.0);
    let bank = pilot.flight.state.bank;
    assert!(bank < -0.5, "bank {bank}");
    let (mut turned, mut sink) = (0.0, 0.0_f32);
    let mut last = heading(&pilot.flight);
    for _ in 0..240 {
        pilot.tick(&keys(0.0, -1.0), &gliding(0.0), DT);
        turned += wrap(heading(&pilot.flight) - last);
        last = heading(&pilot.flight);
        sink = sink.max(-pilot.flight.state.velocity.y);
        let aoa = pilot.flight.state.aoa;
        assert!(aoa.abs() < 0.3, "slipping at {aoa}");
    }
    let rate = turned / 4.0;
    assert!((RATE.0..=RATE.1).contains(&rate), "turns at {rate} rad/s");
    assert!(
        sink < straight * (1.0 + SINK_SHARE),
        "sinks {sink} against {straight}"
    );
}

/// The left key held for 3 s turns the flight path, the nose and the view by the same angle:
/// the camera stays behind the body, and the aim is where the body flies when the key goes.
#[test]
fn a_held_roll_key_carries_the_view_with_the_turn() {
    for roll in [-1.0, 1.0] {
        let mut pilot = settled();
        pilot.run(&keys(0.0, roll), &gliding(0.0), 3.0);
        let (path, nose) = (heading(&pilot.flight), pilot.flight.state.servo.yaw);
        assert!(path * roll < -1.0, "roll {roll}: turned only {path}");
        assert!(
            wrap(pilot.view.x - path).abs() < 0.05,
            "view {}",
            pilot.view
        );
        assert!(wrap(pilot.view.x - nose).abs() < 0.2, "nose {nose}");
        assert_eq!(pilot.stick.aim, pilot.view, "the aim left the view");
    }
}

/// Let go, the wings come level within [`LEVEL_TIME`], one way only, and the glide flies on
/// straight where it was going.
#[test]
fn letting_go_levels_the_wings() {
    for roll in [-1.0, 1.0] {
        let mut pilot = settled();
        pilot.run(&keys(0.0, roll), &gliding(0.0), 3.0);
        let bank = pilot.flight.state.bank;
        for _ in 0..(LEVEL_TIME / DT) as u32 {
            pilot.tick(&keys(0.0, 0.0), &gliding(0.0), DT);
            let now = pilot.flight.state.bank;
            assert!(
                now * bank.signum() >= -PAST_LEVEL,
                "the bank crossed to {now}"
            );
        }
        let level = pilot.flight.state.bank;
        assert!(level.abs() < 0.05, "bank {level}");
        let from = heading(&pilot.flight);
        pilot.run(&keys(0.0, 0.0), &gliding(0.0), 3.0);
        let drift = wrap(heading(&pilot.flight) - from);
        assert!(drift.abs() < 0.1, "still turning by {drift}");
    }
}

/// An aim to one side, as the view gives while the steer button is held, banks the wings that
/// way and turns the flight path onto it, without swinging past it.
#[test]
fn an_aim_to_one_side_banks_and_turns_onto_it() {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    let input = GearInput {
        look: looking(-1.2, -0.35),
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
    }
    let (heading, bank) = (heading(&flight), flight.state.bank);
    assert!((heading + 1.2).abs() < 0.1, "heading {heading}");
    assert!(bank.abs() < 0.05, "wings not level: {bank}");
    assert!(widest <= looking(-1.2, 0.0).x + 0.03, "swung past the aim");
}

/// Flies a glide in a cross wind for `frames` frames with the roll key at `roll`, the stick
/// reading the path through the air, and returns the largest bank on the way.
fn in_wind(gear: &mut Gear, stick: &mut Stick, roll: f32, frames: u32) -> f32 {
    let mut most = 0.0_f32;
    for _ in 0..frames {
        *stick = steer(stick, &gear.in_air(), &keys(0.0, roll), DT).stick;
        let input = GearInput {
            roll,
            wind: Vec3::new(9.0, 0.0, 0.0),
            aim: Some(stick.aim),
            ..gliding(0.0)
        };
        gear.drive(gear.state.velocity, input, DT);
        most = most.max(gear.state.bank.abs());
    }
    most
}

/// In a cross wind a held roll key turns as fast one way as the other: the aim rides the path
/// through the air, which is the one the wings turn, not the track over the ground.
#[test]
fn a_cross_wind_does_not_bias_a_held_turn() {
    let turned = |roll: f32| {
        let mut gear = Gear {
            state: settled().flight.state,
            ..Gear::default()
        };
        let mut stick = Stick::at(Vec2::new(0.0, SLOPE));
        in_wind(&mut gear, &mut stick, 0.0, 600);
        let from = gear.state.servo.yaw;
        let bank = in_wind(&mut gear, &mut stick, roll, 180);
        (wrap(gear.state.servo.yaw - from).abs(), bank)
    };
    let (left, right) = (turned(-1.0), turned(1.0));
    assert!(left.0 > 1.0 && right.0 > 1.0, "{left:?} and {right:?}");
    assert!((left.0 - right.0).abs() < 0.25, "{left:?} and {right:?}");
    assert!((left.1 - right.1).abs() < 0.08, "{left:?} and {right:?}");
}
