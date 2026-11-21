//! Tests of the world markers setting: its three steps, the round trip through a settings file
//! and the files of older versions.

use super::{Settings, WorldMarkers};

/// Every step survives a TOML round trip.
#[test]
fn every_step_round_trips() {
    for level in WorldMarkers::ALL {
        let mut settings = Settings::default();
        settings.gameplay.set_markers(level);
        let text = toml::to_string(&settings).unwrap();
        let back: Settings = toml::from_str(&text).unwrap();
        assert_eq!(back.gameplay.markers(), level, "{text}");
        assert_eq!(back, settings);
    }
}

/// A new settings file shows every marker.
#[test]
fn the_default_is_all() {
    assert_eq!(Settings::default().gameplay.markers(), WorldMarkers::All);
}

/// A file of 0.10 that only has the on or off switch loads and keeps its meaning.
#[test]
fn files_with_only_the_switch_load() {
    let off: Settings = toml::from_str("[gameplay]\nworld_markers = false\n").unwrap();
    assert_eq!(off.gameplay.markers(), WorldMarkers::Off);
    let on: Settings = toml::from_str("[gameplay]\nworld_markers = true\n").unwrap();
    assert_eq!(on.gameplay.markers(), WorldMarkers::All);
    let none: Settings = toml::from_str("[graphics]\nfov = 90\n").unwrap();
    assert_eq!(none.gameplay.markers(), WorldMarkers::All);
}

/// The steps move one at a time and stop at both ends, and Off forgets Minimal.
#[test]
fn the_steps_move_and_stop_at_the_ends() {
    assert_eq!(WorldMarkers::Off.moved(-1), WorldMarkers::Off);
    assert_eq!(WorldMarkers::Off.moved(1), WorldMarkers::Minimal);
    assert_eq!(WorldMarkers::Minimal.moved(1), WorldMarkers::All);
    assert_eq!(WorldMarkers::All.moved(1), WorldMarkers::All);
    let mut settings = Settings::default();
    settings.gameplay.set_markers(WorldMarkers::Minimal);
    assert!(settings.gameplay.world_markers && settings.gameplay.markers_minimal);
    settings.gameplay.set_markers(WorldMarkers::Off);
    assert!(!settings.gameplay.world_markers && !settings.gameplay.markers_minimal);
}
