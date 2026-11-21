//! The words of the course on the prompt line, from `Destination`, `RouteStatus` and the
//! sentences of the Go flow.

use bevy::prelude::*;

use crate::computer::{Block, Keys, RouteStatus, distance, hud_line};

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

/// The action of the prompt while a course runs: the engage key stops it. What the course is doing is not
/// said here, since a status in the slot of an action reads as an action: see `crate::computer::status_line`, shown by the course row of the instruments.
pub const CANCEL: &str = "Cancel the course";

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
fn flying(course: &Course, keys: &Keys) -> Words {
    let name = &course.name;
    let line = hud_line(name, course.route, keys);
    match course.route {
        RouteStatus::Idle => Words {
            ready: Some(match course.gap {
                Some(gap) => format!("Go to {name} - {}", distance(gap)),
                None => format!("Go to {name}"),
            }),
            ..default()
        },
        RouteStatus::Blocked(_) | RouteStatus::Arrived => blocked(line),
        _ => running(CANCEL.to_string()),
    }
}

/// Returns the words of `course` for a player who is `seated` in the ship or not, with the
/// names of the `keys`.
pub fn words(course: &Course, seated: bool, keys: &Keys) -> Words {
    match seated {
        true => flying(course, keys),
        false => blocked(Block::OnFoot.words(keys)),
    }
}
