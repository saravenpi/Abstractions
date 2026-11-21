//! The prompt line shows an action with its key, never the status of the drive: while the drive
//! spools the prompt says "Cancel the course" with the engage key, and the status is a row of the
//! instruments.

use bevy::prelude::KeyCode;

use super::course::{CANCEL, Course, words};
use super::flight::{Flight, Gauge, reading};
use super::prompt::{State, choose};
use crate::computer::{Keys, RouteStatus, status_line};

/// Names of the keys of the tests.
fn keys() -> Keys {
    Keys {
        board: "F".to_string(),
        rise: "Space".to_string(),
        go: "G".to_string(),
    }
}

/// While the drive charges, aligns or runs, the prompt offers to cancel the course on the engage key,
/// and none of the sentences of the status is on the prompt line.
#[test]
fn the_prompt_offers_to_cancel_while_the_drive_runs() {
    for route in [
        RouteStatus::Climbing,
        RouteStatus::Dropping,
        RouteStatus::Aligning(40),
        RouteStatus::Spooling,
        RouteStatus::InWarp,
    ] {
        let course = Course {
            name: "Mars".to_string(),
            route,
            gap: None,
        };
        let said = words(&course, true, &keys());
        let state = State {
            running: said.running.map(|words| (KeyCode::KeyG, words)),
            ..State::default()
        };
        let row = choose(&state).expect("a row");
        assert_eq!(row, (Some(KeyCode::KeyG), CANCEL.to_string()), "{route:?}");
    }
}

/// The status of the course is a row of the instruments, in words that say what the ship does: it holds
/// no key and nothing in it reads as an action.
#[test]
fn the_status_is_a_row_of_the_instruments() {
    let flight = |route| Flight {
        course: Some(status_line("Mars", route)),
        ..Flight::default()
    };
    let spool = reading(Gauge::Course, &flight(RouteStatus::Spooling));
    assert_eq!(spool.as_deref(), Some("Charging the warp drive"));
    let warp = reading(Gauge::Course, &flight(RouteStatus::InWarp));
    assert_eq!(warp.as_deref(), Some("In warp to Mars"));
    assert_eq!(reading(Gauge::Course, &Flight::default()), None);
    assert_eq!(status_line("Mars", RouteStatus::Idle), "");
}
