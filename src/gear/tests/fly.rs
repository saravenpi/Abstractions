//! Wingsuit and boots together: powered flight, and still no hover.

use bevy::prelude::*;

use super::measure::settled_flight;
use super::{DT, Flight, flying, looking};
use crate::gear::{GearInput, Mode};

/// The documented range of the top level speed at sea level on the home planet.
const TOP_SPEED: (f32, f32) = (65.0, 85.0);

/// Looking at the horizon holds height for two minutes.
#[test]
fn level_flight_is_sustainable() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
    let mut low = 0.0_f64;
    for _ in 0..7200 {
        flight.tick(&flying(0.0), DT);
        low = low.min(flight.pos.y);
    }
    assert!(low > -3.0, "sank {low}");
    assert!(flight.pos.y.abs() < 40.0, "drifted to {}", flight.pos.y);
    assert!(flight.state.velocity.y.abs() < 0.5);
}

/// The back key tapped for a quarter second sets a climb that holds: 200 m in well under a
/// minute.
#[test]
fn powered_flight_climbs() {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
    let pull = GearInput {
        pitch: 1.0,
        ..flying(0.0)
    };
    flight.run(&pull, 0.25);
    flight.run(&flying(0.0), 25.0);
    assert!(flight.pos.y > 200.0, "climbed {}", flight.pos.y);
}

/// The level top speed is inside the documented range, set by drag, and a dive held with the
/// forward key half way is faster.
#[test]
fn top_speed_is_in_range() {
    let (level, climb) = settled_flight(0.0);
    assert!(
        (TOP_SPEED.0..=TOP_SPEED.1).contains(&level),
        "level {level}"
    );
    assert!(climb.abs() < 0.5, "not level: {climb}");
    let mut dive = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
    let pushed = GearInput {
        pitch: -0.5,
        ..flying(0.0)
    };
    dive.run(&pushed, 30.0);
    assert!(
        dive.state.airspeed > level + 15.0,
        "dive {}",
        dive.state.airspeed
    );
}

/// At rest the boosters and wings together cannot hold the weight, wherever the pilot looks.
#[test]
fn hovering_is_impossible() {
    for degrees in (-80..=90).step_by(10) {
        let mut flight = Flight::new(Mode::Fly, Vec3::ZERO);
        let input = GearInput {
            pitch: 1.0,
            ..flying((degrees as f32).to_radians())
        };
        flight.tick(&input, DT);
        let fall = -flight.state.velocity.y / DT;
        assert!(fall > 0.15 * 22.0, "look {degrees}: falls at only {fall}");
    }
}

/// Looking straight up from rest never rises above the start.
#[test]
fn straight_up_from_rest_never_rises() {
    let mut flight = Flight::new(Mode::Fly, Vec3::ZERO);
    let input = GearInput {
        pitch: 1.0,
        look: Vec3::Y,
        ..flying(0.0)
    };
    for _ in 0..300 {
        flight.tick(&input, DT);
        assert!(flight.pos.y <= 0.0, "rose to {}", flight.pos.y);
    }
}

/// Jump, deploy, boost: the body never drops below the ground it left and is flying fast and
/// prone within ten seconds.
#[test]
fn take_off_from_a_jump() {
    for start in [Vec3::new(0.0, 7.0, 0.0), Vec3::new(3.0, 7.0, -8.5)] {
        let mut flight = Flight::new(Mode::Off, start);
        for _ in 0..600 {
            flight.tick(&flying(0.0), DT);
            assert!(flight.pos.y >= 0.0, "dropped to {}", flight.pos.y);
        }
        assert_eq!(flight.state.mode, Mode::Fly);
        assert!(flight.state.airspeed > 60.0 && flight.state.prone > 0.99);
    }
}

/// The view steers nothing under power either: a slow body looking far to one side flies on
/// exactly as one looking ahead.
#[test]
fn powered_flight_ignores_the_view() {
    let ahead = |input: GearInput| {
        let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -6.0));
        flight.run(&input, 1.5);
        flight.state
    };
    let aside = GearInput {
        look: looking(1.2, 0.4),
        aim: Some(Vec2::new(1.2, 0.4)),
        ..flying(0.0)
    };
    assert_eq!(ahead(aside), ahead(flying(0.0)));
}
