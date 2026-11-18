//! The words of the header and the footer: where the ship is, what the main button does,
//! and the course with its distance and a rough time to arrival.

use super::format::distance;
use super::route::{RouteStatus, Target};
use super::view::Act;
use crate::saves::play_time;

/// What the breadcrumb says when the ship is inside no known galaxy.
pub const NOWHERE: &str = "Deep space";

/// Slowest speed in metres per second a time to arrival is worked out from.
pub const SLOWEST: f64 = 1.0;

/// Returns the breadcrumb of the names of what the ship is inside, from the galaxy down to the
/// planet, or [`NOWHERE`] when it is inside nothing.
pub fn crumbs(names: &[String]) -> String {
    match names {
        [] => NOWHERE.to_string(),
        names => names.join(" > "),
    }
}

/// Returns what the main button does for the target of the selected row, the course that is
/// set and the state of the route: set the course to a row that is not the course yet, else
/// engage the autopilot on a course it does not fly yet, else clear the course. Nothing when
/// there is no row and no course.
pub fn primary(cursor: Option<Target>, course: Option<Target>, status: RouteStatus) -> Option<Act> {
    match (cursor, course) {
        (Some(_), _) if cursor != course => Some(Act::Set),
        (_, None) => None,
        _ if status == RouteStatus::Idle => Some(Act::Engage),
        _ => Some(Act::Clear),
    }
}

/// Returns what the main button says for `act`, with `name` the name of the selected row.
pub fn words(act: Act, name: &str) -> String {
    match act {
        Act::Set => format!("Set course to {name}"),
        Act::Engage => "Engage autopilot".to_string(),
        Act::Clear => "Clear course".to_string(),
        Act::Home => "Set course home".to_string(),
    }
}

/// Returns a rough time to cover `gap` metres at `speed` metres per second, or nothing while
/// the ship is too slow for the number to mean anything.
pub fn arrival(gap: f64, speed: f64) -> Option<String> {
    (speed >= SLOWEST).then(|| format!("about {}", play_time(gap / speed)))
}

/// Returns the course line of the footer: the name of the destination, how far it is, the
/// time to get there at `speed` metres per second, and the state of the route.
pub fn course(to: Option<(&str, f64)>, speed: f64, status: RouteStatus) -> String {
    let Some((name, gap)) = to else {
        return "No course set. Select a place, then set course".to_string();
    };
    let eta = arrival(gap, speed).map_or(String::new(), |eta| format!(" - {eta}"));
    format!(
        "Course {name} - {}{eta} - {}",
        distance(gap),
        status.label()
    )
}
