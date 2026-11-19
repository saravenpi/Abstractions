//! The words of the header and the footer: where the ship is, what the main button does,
//! the course with its distance and a rough time to arrival, and the action line of the course.

use super::format::distance;
use bevy::prelude::*;

use super::route::RouteStatus;
use crate::saves::play_time;
use crate::ui::Icon;

/// What a button or a key of the map does to the course.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Set the course to the selected row.
    Set,
    /// Hand the course to the autopilot.
    Engage,
    /// Forget the course.
    Clear,
    /// Set the course to the next place on the way home.
    Home,
}

/// What the course line says while no course is set.
pub const NO_COURSE: &str = "Double-click a place or press Enter to set a course";

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

/// Returns a rough time to cover `gap` metres at `speed` metres per second, or nothing while
/// the ship is too slow for the number to mean anything.
pub fn arrival(gap: f64, speed: f64) -> Option<String> {
    (speed >= SLOWEST).then(|| format!("about {}", play_time(gap / speed)))
}

/// Returns the course line of the footer: the name of the destination, how far it is, the
/// time to get there at `speed` metres per second, and the state of the route.
pub fn course(to: Option<(&str, f64)>, speed: f64, status: RouteStatus) -> String {
    let Some((name, gap)) = to else {
        return NO_COURSE.to_string();
    };
    let eta = arrival(gap, speed).map_or(String::new(), |eta| format!(" - {eta}"));
    format!(
        "Course {name} - {}{eta} - {}",
        distance(gap),
        status.label()
    )
}

/// Returns the action line of a course to `name` in the state `status`: whether the engage
/// key does something now, and what the line says. The engage key is shown only while the
/// autopilot can take the course; otherwise the line says the first reason it cannot, or
/// how far along the route is.
pub fn action(name: &str, status: RouteStatus) -> (bool, String) {
    match status {
        RouteStatus::Idle => (true, format!("Engage autopilot to {name}")),
        RouteStatus::Blocked(block) => (false, block.words()),
        RouteStatus::Dropping => (false, "Dropping out of warp".to_string()),
        RouteStatus::Aligning(done) => (false, format!("Aligning {done}%")),
        RouteStatus::Spooling => (false, "Spooling the warp drive".to_string()),
        RouteStatus::InWarp => (false, format!("In warp to {name}")),
        RouteStatus::Arrived => (false, format!("Arrived at {name}")),
    }
}

/// What the action line of the course offers: a button that runs an act, with its icon and
/// its words, or words alone while there is nothing to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Offer {
    /// A button that runs this act.
    Button(Act, Icon, String),
    /// Words alone.
    Words(String),
}

/// Returns what the action line offers: setting the course while the selection is not the
/// course, engaging the autopilot while the route to the course named `course` is idle,
/// clearing it once arrived, and otherwise what the route is doing or why it cannot start.
pub fn offer(course: Option<&str>, set: bool, status: RouteStatus) -> Offer {
    if set {
        return Offer::Button(Act::Set, Icon::MapPin, "Set course".to_string());
    }
    let Some(name) = course else {
        return Offer::Words(String::new());
    };
    match status {
        RouteStatus::Idle => Offer::Button(Act::Engage, Icon::Send, action(name, status).1),
        RouteStatus::Arrived => Offer::Button(Act::Clear, Icon::Close, "Clear course".to_string()),
        _ => Offer::Words(action(name, status).1),
    }
}
