//! The words of the course: what a button or a key does, the course line with its distance and
//! a rough time to arrival, and the sentences of the card and the HUD that say what Go does
//! now, why it cannot, or how far along the route is.

use crate::ui::units::distance;
use bevy::prelude::*;

use super::route::{Block, Keys, RouteStatus, Target};
use crate::ui::units::duration;

/// What a button or a key of the map does to the course.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// Send the ship to the selected place, or stop the route it flies.
    Go,
    /// Fly home in one command.
    Home,
    /// Show or hide the rarer facts of the card.
    More,
}

/// Slowest speed in metres per second a time to arrival is worked out from.
pub const SLOWEST: f64 = 1.0;

/// What the card says while no place is selected.
pub const NO_PLACE: &str = "Click a place to see how to get there.";

/// What the card says when the place is where the player is.
pub const HERE: &str = "You are here.";

/// What the card says when the autopilot can take the course but no time can be told yet.
pub const READY: &str = "The autopilot flies there.";

/// Returns a rough time to cover `gap` metres at `speed` metres per second, or nothing while
/// the ship is too slow for the number to mean anything.
pub fn arrival(gap: f64, speed: f64) -> Option<String> {
    (speed >= SLOWEST).then(|| format!("about {}", duration(gap / speed)))
}

/// Returns the course line of the footer: the name of the destination, how far it is and the
/// time to get there at `speed` metres per second. Empty without a destination.
pub fn course(to: Option<(&str, f64)>, speed: f64) -> String {
    let Some((name, gap)) = to else {
        return String::new();
    };
    let eta = arrival(gap, speed).map_or(String::new(), |eta| format!(" - {eta}"));
    format!("Course {name} - {}{eta}", distance(gap))
}

/// Returns true while the route is flown: the ship climbs, turns, charges or warps.
pub fn running(status: RouteStatus) -> bool {
    use RouteStatus::{Aligning, Climbing, Dropping, InWarp, Spooling};
    matches!(
        status,
        Climbing | Dropping | Aligning(_) | Spooling | InWarp
    )
}

/// Returns the sentence for a route in the state `status` to `name`: what the ship does now,
/// why the drive cannot take the course, or that it arrived. Empty while the route is idle.
pub fn hud_line(name: &str, status: RouteStatus, keys: &Keys) -> String {
    match status {
        RouteStatus::Idle => String::new(),
        RouteStatus::Blocked(block) => block.words(keys),
        RouteStatus::Climbing => "Climbing out of the air.".to_string(),
        RouteStatus::Dropping => "Slowing down.".to_string(),
        RouteStatus::Aligning(_) => format!("Turning toward {name}."),
        RouteStatus::Spooling => "Charging the warp drive.".to_string(),
        RouteStatus::InWarp => format!("Warping to {name}. {} stops.", keys.go),
        RouteStatus::Arrived => format!("Arrived at {name}."),
    }
}

/// What the card of a place needs to say what Go does there.
#[derive(Debug, Clone, Copy)]
pub struct Ask<'a> {
    /// Name of the place.
    pub name: &'a str,
    /// Whether the place is the course.
    pub ours: bool,
    /// Whether the player is at the place.
    pub here: bool,
    /// Whether the player sits in the ship.
    pub seated: bool,
    /// The state of the route.
    pub status: RouteStatus,
    /// The time to get there, such as `about 4 min`, while the ship moves.
    pub eta: Option<&'a str>,
    /// The names of the keys the sentences mention.
    pub keys: &'a Keys,
}

/// Returns `words` with its first letter in capitals and a full stop at its end.
fn sentence(words: &str) -> String {
    let mut letters = words.chars();
    let first = letters
        .next()
        .map_or(String::new(), |c| c.to_uppercase().to_string());
    format!("{first}{}.", letters.as_str())
}

/// Returns whether Go can start now on the place `ask` describes, and the one sentence under
/// its button: why it cannot and what to do, what the route does, or how long the trip takes.
/// While the route to the place is flown the sentence says so, and the button reads Stop.
pub fn card_line(ask: &Ask) -> (bool, String) {
    let flown = !matches!(ask.status, RouteStatus::Idle | RouteStatus::Blocked(_));
    if ask.ours && flown {
        return (false, hud_line(ask.name, ask.status, ask.keys));
    }
    if ask.here {
        return (false, HERE.to_string());
    }
    if !ask.seated {
        return (false, Block::OnFoot.words(ask.keys));
    }
    match ask.status {
        RouteStatus::Blocked(block) => (false, block.words(ask.keys)),
        _ => (true, ask.eta.map_or(READY.to_string(), sentence)),
    }
}

/// What a press of the Go key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Press {
    /// Set the course to the place and hand it to the autopilot.
    Go(Target),
    /// Stop the route the autopilot flies.
    Stop,
    /// Nothing: no place, the player is on foot, or the drive is blocked.
    Nothing,
}

/// Returns what the Go key does for the place `pick`, or for the course when nothing is picked.
/// `can` is true when the player sits in the ship and is not at the place. It stops a route to
/// the place that is flown, and starts one otherwise, unless something blocks the drive.
pub fn press(
    pick: Option<Target>,
    course: Option<Target>,
    status: RouteStatus,
    can: bool,
) -> Press {
    let Some(target) = pick.or(course) else {
        return Press::Nothing;
    };
    match (running(status), course == Some(target)) {
        (true, true) => Press::Stop,
        _ if !can || matches!(status, RouteStatus::Blocked(_)) => Press::Nothing,
        _ => Press::Go(target),
    }
}
