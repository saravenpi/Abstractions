//! Screen tests: the wake-up script and its timing, and what counts as a new world.

use super::loading::Arrival;
use super::wake::{FADE, STATUS_DELAY, line_time, script, timeline};
use crate::saves::{ActiveWorld, WorldEntry, WorldSave};
use crate::universe::{HOME_PLANET_NAME, HOME_STAR_NAME};

/// Returns the active world of a save played for `seconds`.
fn active(seconds: f64) -> ActiveWorld {
    ActiveWorld(WorldEntry {
        id: "isle".to_string(),
        save: WorldSave {
            seed: 7,
            play_seconds: seconds,
            ..WorldSave::default()
        },
    })
}

/// A world never played is new and arrives on the home planet; a played one is a return.
#[test]
fn a_world_never_played_is_fresh() {
    let new = Arrival::of(Some(&active(0.0)), 7);
    assert!(new.fresh);
    assert_eq!(
        (new.planet.as_str(), new.star.as_str()),
        (HOME_PLANET_NAME, HOME_STAR_NAME)
    );
    assert!(!Arrival::of(Some(&active(125.0)), 7).fresh);
    assert!(!Arrival::of(None, 7).fresh);
}

/// A new world wakes up in five lines that end on its location, within four to six seconds.
#[test]
fn a_new_world_gets_the_full_wake_up() {
    let lines = script(&Arrival::of(Some(&active(0.0)), 7));
    let labels: Vec<&str> = lines.iter().map(|(label, _)| *label).collect();
    assert_eq!(
        labels,
        [
            "QUANTUM LINK",
            "OPERATOR",
            "UNIT 01",
            "COMPANION UNIT",
            "LOCATION"
        ]
    );
    assert_eq!(lines[0].1, "ESTABLISHED");
    assert_eq!(lines[4].1, "KODYA, VEREYA");
    let (fade_at, end) = timeline(lines.len());
    assert!((4.0..=6.0).contains(&end), "{end}");
    assert!((end - fade_at - FADE).abs() < 1e-6);
    assert!(fade_at > line_time(4) + STATUS_DELAY);
    assert!(line_time(0) < line_time(1));
}

/// A resumed world gets one short line.
#[test]
fn a_resumed_world_gets_one_line() {
    let lines = script(&Arrival::of(Some(&active(125.0)), 7));
    assert_eq!(lines, vec![("LINK", "RESTORED".to_string())]);
    let (_, end) = timeline(lines.len());
    assert!(end < 2.5, "{end}");
}
