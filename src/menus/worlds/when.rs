//! Human readable facts of a saved world: when it was last played and where the player is.

use crate::saves::WorldSave;

/// Seconds in a minute, an hour, a day, a week, a month of 30 days and a year of 365 days.
const SPANS: [(u64, &str); 6] = [
    (31_536_000, "year"),
    (2_592_000, "month"),
    (604_800, "week"),
    (86_400, "day"),
    (3600, "hour"),
    (60, "minute"),
];

/// Describes how long ago `then` was, relative to `now`, both in unix seconds: "just now",
/// "5 minutes ago", "2 hours ago", "yesterday", "3 weeks ago". A time of zero reads "never".
pub fn relative(now: u64, then: u64) -> String {
    if then == 0 {
        return "never".to_string();
    }
    let seconds = now.saturating_sub(then);
    let Some((span, unit)) = SPANS.iter().find(|(span, _)| seconds >= *span) else {
        return "just now".to_string();
    };
    match (seconds / span, *unit) {
        (1, "day") => "yesterday".to_string(),
        (1, unit) => format!("1 {unit} ago"),
        (count, unit) => format!("{count} {unit}s ago"),
    }
}

/// Returns the planet and the system the player of `save` is on, as "planet, star".
///
/// A new world and a world from before v0.6 have no saved place: they start on the home planet.
pub fn place_name(save: &WorldSave) -> String {
    let universe = save.universe();
    let body = save.body(&universe);
    let planet = universe.planet(body).name;
    let star = universe.system(body.star).star.name;
    format!("{planet}, {star}")
}
