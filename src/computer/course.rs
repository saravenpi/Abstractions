//! The words of the header and the footer: where the ship is, which actions the footer
//! offers, and the course with its distance and a rough time to arrival.

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

/// Returns true when the footer offers `act`, for the target of the row under the cursor and
/// the course that is set. Only one of setting the course and engaging shows at a time.
pub fn offered(act: Act, cursor: Option<Target>, course: Option<Target>) -> bool {
    match act {
        Act::Set => cursor.is_some() && cursor != course,
        Act::Engage => course.is_some() && (cursor == course || cursor.is_none()),
        Act::Clear => course.is_some(),
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
        return "No course set. Pick a row and press Enter".to_string();
    };
    let eta = arrival(gap, speed).map_or(String::new(), |eta| format!(" - {eta}"));
    format!(
        "Course {name} - {}{eta} - {}",
        distance(gap),
        status.label()
    )
}
