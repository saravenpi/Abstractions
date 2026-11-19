//! The words of the course: what a button or a key does to it, the course line with its
//! distance and a rough time to arrival, what the route is doing and the progress line.

use super::format::distance;
use bevy::prelude::*;

use super::route::RouteStatus;
use crate::saves::play_time;

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
pub const NO_COURSE: &str = "Right-click a place or press Enter to set a course";

/// Slowest speed in metres per second a time to arrival is worked out from.
pub const SLOWEST: f64 = 1.0;

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

/// Returns the progress line under the map for a course to `name` in the state `status`:
/// how far along the route is while it is flown, and nothing otherwise, since the card of the
/// place offers the buttons and says why the drive cannot engage.
pub fn progress(name: &str, status: RouteStatus) -> String {
    match status {
        RouteStatus::Idle | RouteStatus::Blocked(_) | RouteStatus::Arrived => String::new(),
        _ => action(name, status).1,
    }
}
