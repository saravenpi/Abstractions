//! The gear in the air of other planets: none, thin, home and thick.

use bevy::prelude::*;

use super::{Flight, braking, flying, gliding};
use crate::gear::{GearInput, Mode, descent_target};
use crate::universe::PlanetRecord;
use crate::wind::{PRESSURES, samples, thickness};

/// Returns `input` in the air of a planet with `pressure` times the home pressure.
fn under(pressure: f64, input: GearInput) -> GearInput {
    let record = PlanetRecord {
        pressure,
        ..PlanetRecord::TEMPERATE
    };
    GearInput {
        air: thickness(&record) as f32,
        ..input
    }
}

/// The air of the home planet is the air the gear was tuned in: the default input, and the
/// same flight bit for bit when the pressure says 1.
#[test]
fn home_air_changes_nothing() {
    assert_eq!(under(1.0, GearInput::default()), GearInput::default());
    assert_eq!(thickness(&samples()[1]), 1.0);
    for (mode, input) in [
        (Mode::Glide, gliding(-0.2)),
        (Mode::Fly, flying(0.3)),
        (Mode::BootFall, braking()),
        (Mode::Fall, GearInput::default()),
    ] {
        let start = Vec3::new(3.0, -9.0, -42.0);
        let (mut plain, mut home) = (Flight::new(mode, start), Flight::new(mode, start));
        plain.run(&input, 12.0);
        home.run(&under(1.0, input), 12.0);
        assert_eq!(plain.state, home.state, "{mode:?}");
    }
}

/// On a planet without air a drop with the wings open or folded is a ballistic fall, whatever
/// the sticks ask: gravity alone, nothing felt, and every number finite.
#[test]
fn without_air_a_drop_is_ballistic() {
    for pressure in [0.0, 0.005, samples()[5].pressure] {
        for (mode, input) in [
            (Mode::Glide, gliding(0.4)),
            (Mode::Fall, GearInput::default()),
        ] {
            let start = Vec3::new(12.0, -30.0, -55.0);
            let mut flight = Flight::new(mode, start);
            let sticks = GearInput {
                pitch: 1.0,
                roll: -1.0,
                aim: Some(Vec2::new(0.8, 0.5)),
                ..under(pressure, input)
            };
            flight.run(&sticks, 10.0);
            let off = flight.state.velocity - (start - Vec3::Y * 220.0);
            assert!(off.length() < 0.05, "{mode:?} off by {off}");
            assert!(flight.state.load.abs() < 1e-3 && flight.twist.is_finite());
        }
    }
}

/// On a planet without air the boots are the only brake, and they brake as at home: a fall
/// eases to the same descent speed with the wings folded or open, and the push never runs
/// away sideways for want of drag.
#[test]
fn the_boots_brake_without_air() {
    let target = descent_target(22.0);
    for input in [braking(), flying(0.0), flying(0.6)] {
        let mode = if input.deploy {
            Mode::Fly
        } else {
            Mode::BootFall
        };
        let mut flight = Flight::new(mode, Vec3::new(0.0, -60.0, -20.0));
        flight.run(&under(0.0, input), 30.0);
        let v = flight.state.velocity;
        assert!((v.y + target).abs() < 0.3, "{mode:?} sinks at {}", -v.y);
        assert!(v.with_y(0.0).length() < 12.0, "{mode:?} runs at {v}");
        assert!(flight.pos.y < 0.0 && flight.kick < 3.0);
    }
}

/// Returns the airspeed and the sink rate a glide settles at in air of `pressure`, looking
/// 15 degrees down, averaged over its last ten seconds.
fn glide(pressure: f64) -> (f32, f32) {
    let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -12.0, -36.0));
    let input = under(pressure, gliding(-0.26));
    flight.run(&input, 50.0);
    let (mut speed, mut sink) = (0.0, 0.0);
    for _ in 0..600 {
        flight.tick(&input, super::DT);
        speed += flight.state.airspeed / 600.0;
        sink -= flight.state.velocity.y / 600.0;
    }
    (speed, sink)
}

/// In thin air a glide is fast and sinks fast, in thick air it floats: airspeed and sink rate
/// both fall as the air thickens, and the wings still carry in every air there is.
#[test]
fn thin_air_glides_fast_and_thick_air_floats() {
    let [thin, home, thick] = [0.3, 1.0, 3.0].map(glide);
    assert!(
        thin.0 > 1.3 * home.0 && home.0 > 1.3 * thick.0,
        "{thin:?} {home:?} {thick:?}"
    );
    assert!(
        thin.1 > 1.3 * home.1 && home.1 > 1.3 * thick.1,
        "{thin:?} {home:?} {thick:?}"
    );
    for (speed, sink) in [thin, home, thick] {
        assert!(
            sink > 0.0 && speed / sink > 1.5,
            "speed {speed} sink {sink}"
        );
    }
}

/// The air slows a flat fall at 40 m/s with the wings open more the thicker it is, at every
/// pressure the planets have, and not at all without air.
#[test]
fn drag_grows_with_the_pressure() {
    let slowed = |pressure: f64| {
        let mut flight = Flight::new(Mode::Glide, Vec3::new(0.0, -40.0, 0.0));
        let input = GearInput {
            gravity: 1e-3,
            look: Vec3::ZERO,
            ..under(pressure, gliding(0.0))
        };
        flight.run(&input, 1.0);
        flight.state.velocity.length()
    };
    let left = PRESSURES.map(slowed);
    assert!((left[0] - 40.0).abs() < 0.01 && (left[1] - 40.0).abs() < 0.01);
    assert!(left.windows(2).all(|pair| pair[0] >= pair[1]), "{left:?}");
    assert!(left[2] < 39.0 && left[4] < left[3] - 1.0, "{left:?}");
    assert!(left.iter().all(|speed| speed.is_finite()));
}

/// Powered flight tops out lower in thick air, and in thin air it does not run away: the
/// spread wings hand the thrust they cannot carry back to the brake.
#[test]
fn powered_flight_stays_bounded_in_every_air() {
    let top = |pressure: f64| {
        let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -40.0));
        flight.run(&under(pressure, flying(0.0)), 60.0);
        assert!(flight.state.velocity.is_finite() && flight.kick < 3.0);
        flight.state.velocity.length()
    };
    let speeds = PRESSURES.map(top);
    assert!(speeds[4] < 0.75 * speeds[3], "{speeds:?}");
    assert!(
        speeds.iter().all(|speed| *speed < 1.2 * speeds[3]),
        "{speeds:?}"
    );
}
