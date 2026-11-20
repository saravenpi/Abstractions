//! The words of the course on the prompt line, read from what exists today: `Destination`,
//! `RouteStatus` and `Block::words`.
//!
//! This is the one adapter between the HUD and the Go flow. When the Go flow lands, replace the
//! body of [`words`] by a call of `computer::hud_line` and keep its signature.

use bevy::prelude::*;

use crate::computer::{RouteStatus, distance};

/// A course as the prompt line needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct Course {
    /// Name of the destination.
    pub name: String,
    /// How far along the route the ship is.
    pub route: RouteStatus,
    /// Distance to the destination in metres, when known.
    pub gap: Option<f64>,
}

/// What the course says on the prompt line. At most one field is set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Words {
    /// A sentence with no key: why the course cannot go, or that it arrived.
    pub block: Option<String>,
    /// A course that runs: what it is doing now. The engage key stops it.
    pub running: Option<String>,
    /// A course ready to go. The engage key starts it.
    pub ready: Option<String>,
}

/// Returns a words value with only `sentence` as its running line.
fn running(sentence: String) -> Words {
    Words {
        running: Some(sentence),
        ..default()
    }
}

/// Returns a words value with only `sentence` as its block line.
fn blocked(sentence: String) -> Words {
    Words {
        block: Some(sentence),
        ..default()
    }
}

/// Returns the words of a course the ship, seated, can start or is flying.
fn flying(course: &Course) -> Words {
    let name = &course.name;
    match course.route {
        RouteStatus::Idle => Words {
            ready: Some(match course.gap {
                Some(gap) => format!("Go to {name} - {}", distance(gap)),
                None => format!("Go to {name}"),
            }),
            ..default()
        },
        RouteStatus::Blocked(block) => blocked(block.words()),
        RouteStatus::Climbing => running("Climbing out of the air.".to_string()),
        RouteStatus::Dropping => running("Slowing down.".to_string()),
        RouteStatus::Aligning(_) => running(format!("Turning toward {name}.")),
        RouteStatus::Spooling => running("Charging the warp drive.".to_string()),
        RouteStatus::InWarp => running(format!("Warping to {name}")),
        RouteStatus::Arrived => blocked(format!("Arrived at {name}.")),
    }
}

/// Returns the words of `course` for a player who is `seated` in the ship, with the board key.
pub fn words(
    course: &Course,
    seated: bool,
    board: KeyCode,
    key_name: impl Fn(KeyCode) -> String,
) -> Words {
    match seated {
        true => flying(course),
        false => blocked(format!(
            "Board your ship first: walk to the hatch and press {}.",
            key_name(board)
        )),
    }
}
