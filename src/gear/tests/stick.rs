//! The stick law: the keys always move the aim, the view only while the steer button is held.

use bevy::prelude::*;

use super::pilot::{Pilot, keys};
use super::{DT, Flight, flying, gliding};
use crate::gear::aim::wrap;
use crate::gear::stick::{Hands, Stick, steer};
use crate::gear::tune::{LEASH, STICK_RATE};
use crate::gear::{FlightState, Mode};

/// Returns a pilot on a fast powered flight, level and settled.
fn cruising() -> Pilot {
    let mut flight = Flight::new(Mode::Fly, Vec3::new(0.0, 0.0, -70.0));
    flight.run(&flying(0.0), 5.0);
    Pilot::on(flight, 0.0)
}

/// Returns hands that move the mouse by `mouse` radians a frame, with the steer button `held`.
fn mouse(mouse: Vec2, held: bool) -> Hands {
    Hands {
        mouse,
        held,
        ..Hands::default()
    }
}

/// With the steer button up the mouse only looks around: the view turns, the aim does not,
/// and the body flies on exactly as one whose mouse is left alone.
#[test]
fn the_mouse_steers_nothing_with_the_button_up() {
    let (mut looking, mut still) = (cruising(), cruising());
    looking.run(&mouse(Vec2::new(0.01, -0.004), false), &flying(0.0), 2.0);
    still.run(&keys(0.0, 0.0), &flying(0.0), 2.0);
    assert!(
        looking.view.x > 1.0 && looking.view.y < -0.4,
        "{}",
        looking.view
    );
    assert_eq!(looking.stick.aim, still.stick.aim);
    assert_eq!(looking.flight.state, still.flight.state);
}

/// With the button held the aim is the view: the mouse turns both alike and the body follows.
#[test]
fn the_view_aims_the_flight_while_the_button_is_held() {
    let mut pilot = cruising();
    pilot.run(&mouse(Vec2::new(0.004, 0.002), true), &flying(0.0), 1.5);
    assert_eq!(pilot.stick.aim, pilot.view);
    assert!(
        pilot.stick.aim.x > 0.3 && pilot.stick.aim.y > 0.15,
        "aim {}",
        pilot.stick.aim
    );
    pilot.run(&mouse(Vec2::ZERO, true), &flying(0.0), 4.0);
    let state = pilot.flight.state;
    assert!(
        wrap(state.servo.yaw - pilot.stick.aim.x).abs() < 0.1,
        "{state:?}"
    );
    assert!(state.velocity.y > 5.0, "not climbing: {}", state.velocity);
}

/// Pressing the button while the view points away brings the aim to the view at the stick
/// rate, never in one frame, and letting go leaves the aim exactly where it was: no snap, no
/// easing back.
#[test]
fn pressing_brings_the_aim_over_and_releasing_leaves_it() {
    let mut pilot = cruising();
    pilot.run(&mouse(Vec2::new(0.04, 0.0), false), &flying(0.0), 0.5);
    let away = pilot.view.x;
    assert!(away > 1.0 && pilot.stick.aim.x == 0.0, "view {away}");
    let mut last = pilot.stick.aim;
    for frame in 0..30 {
        pilot.tick(&mouse(Vec2::ZERO, true), &flying(0.0), DT);
        let moved = (pilot.stick.aim - last).length();
        assert!(moved <= STICK_RATE * DT + 1e-5, "frame {frame}: {moved}");
        assert!(moved > 0.0, "frame {frame}: the aim stood still");
        last = pilot.stick.aim;
    }
    assert!(pilot.stick.aim.x < away - 1e-3, "the aim is already there");
    pilot.run(&mouse(Vec2::new(-0.01, 0.003), false), &flying(0.0), 2.0);
    assert_eq!(pilot.stick.aim, last, "the aim moved after the release");
    pilot.run(&mouse(Vec2::ZERO, true), &flying(0.0), 3.0);
    assert_eq!(
        pilot.stick.aim, pilot.view,
        "the aim never reached the view"
    );
}

/// The pitch keys turn the aim and the view together with the button up or held, and the
/// aim stays where a let go key left it.
#[test]
fn the_pitch_keys_turn_the_aim_and_the_view() {
    for held in [false, true] {
        let mut pilot = cruising();
        let hands = Hands {
            held,
            ..keys(1.0, 0.0)
        };
        pilot.run(&hands, &flying(0.0), 0.25);
        let up = pilot.stick.aim.y;
        assert!((up - 0.25 * STICK_RATE).abs() < 1e-3, "aim {up}");
        assert_eq!(pilot.stick.aim, pilot.view);
        pilot.run(&mouse(Vec2::ZERO, held), &flying(0.0), 3.0);
        assert_eq!(pilot.stick.aim.y, up, "the aim eased back");
    }
}

/// Neither the keys nor the mouse lead the aim farther in pitch than the leash from the
/// flight frame, and an aim already farther is not dragged back.
#[test]
fn the_aim_is_leashed_to_the_flight_frame_one_way() {
    let state = FlightState {
        mode: Mode::Glide,
        wings: 1.0,
        velocity: Vec3::new(0.0, -5.0, -30.0),
        ..FlightState::default()
    };
    let pulled = |aim: f32, hands: Hands| {
        let moved = steer(&Stick::at(Vec2::new(0.0, aim)), &state, &hands, 0.5);
        (moved.stick.aim.y, moved.turn.y)
    };
    let key = pulled(0.6, keys(1.0, 0.0)).0;
    assert!((key - LEASH).abs() < 1e-6, "aim {key}");
    let hand = Hands {
        view: Vec2::new(0.0, 0.6),
        ..mouse(Vec2::new(0.0, 2.0), true)
    };
    let led = pulled(0.6, hand);
    assert!((led.0 - LEASH).abs() < 1e-6 && (led.1 - 0.3).abs() < 1e-6);
    let far = pulled(1.3, keys(1.0, 0.0)).0;
    assert_eq!(far, 1.3, "an aim past the leash moved");
    let back = pulled(1.3, keys(-1.0, 0.0)).0;
    assert!(back < 1.3, "an aim past the leash cannot come back");
}

/// The law gives the same aim at 30 and at 144 frames a second.
#[test]
fn the_stick_does_not_depend_on_the_frame_rate() {
    let flown = |rate: f32| {
        let mut pilot = Pilot::on(Flight::new(Mode::Glide, Vec3::new(0.0, -8.0, -32.0)), -0.2);
        let hands = Hands {
            mouse: Vec2::new(0.6, 0.2) / rate,
            held: true,
            ..keys(-1.0, 0.0)
        };
        for _ in 0..(1.5 * rate) as u32 {
            pilot.tick(&hands, &gliding(0.0), 1.0 / rate);
        }
        pilot.stick.aim
    };
    let (slow, fast) = (flown(30.0), flown(144.0));
    assert!((slow - fast).length() < 0.03, "{slow} against {fast}");
}
