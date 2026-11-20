//! Tests of the first-time hints: each is offered in its moment, once, in a fixed order, and the
//! bit that says it was done survives a settings round trip.

use bevy::prelude::KeyCode;

use super::context::Context;
use super::hints::{Clock, Hint, HOLD, done, due, finished, with};
use crate::settings::{Keymap, Settings};

/// A clock that has waited `seated` seconds in the ship and `afoot` on the ground.
fn waited(seated: f32, afoot: f32) -> Clock {
    Clock {
        seated,
        afoot,
        ..Clock::default()
    }
}

/// The map hint waits five seconds in the ship, then shows, and never shows again once done.
#[test]
fn the_map_hint_shows_once() {
    assert_eq!(due(0, &waited(4.0, 0.0), Context::ShipLanded), None);
    assert_eq!(due(0, &waited(6.0, 0.0), Context::ShipLanded), Some(Hint::Map));
    assert_eq!(due(Hint::Map.bit(), &waited(6.0, 0.0), Context::ShipLanded), None);
    assert_eq!(due(0, &waited(6.0, 0.0), Context::OnFoot), None);
}

/// The pin hint waits for a minute on the ground and for the map to have been opened.
#[test]
fn the_pin_hint_needs_the_map_first() {
    let clock = waited(0.0, 61.0);
    assert_ne!(due(0, &clock, Context::OnFoot), Some(Hint::Pin));
    let seen = with(0, Hint::Map);
    assert_eq!(due(with(seen, Hint::Minimap), &clock, Context::OnFoot), Some(Hint::Pin));
}

/// A hint given up for the session is not offered again before the next session, and at most one
/// shows at a time.
#[test]
fn a_hushed_hint_gives_way_to_the_next() {
    let mut clock = waited(20.0, 0.0);
    assert_eq!(due(0, &clock, Context::ShipLanded), Some(Hint::Map));
    clock.hushed = Hint::Map.bit();
    assert_eq!(due(0, &clock, Context::ShipLanded), Some(Hint::Assist));
    assert!(HOLD > 0.0);
}

/// The key of a hint finishes it, and the computer key finishes the map hint.
#[test]
fn the_hint_key_finishes_the_hint() {
    let keys = Keymap::default();
    assert_eq!(finished(&keys, |key| key == keys.waypoint), Some(Hint::Pin));
    assert_eq!(finished(&keys, |key| key == keys.computer), Some(Hint::Map));
    assert_eq!(finished(&keys, |key| key == KeyCode::KeyP), None);
}

/// The hints that were done persist through a settings round trip.
#[test]
fn hint_bits_survive_a_settings_round_trip() {
    let mut settings = Settings::default();
    assert_eq!(settings.hints, 0);
    settings.hints = with(with(0, Hint::Map), Hint::Pin);
    let text = toml::to_string(&settings).unwrap();
    let back: Settings = toml::from_str(&text).unwrap();
    assert!(done(back.hints, Hint::Map) && done(back.hints, Hint::Pin));
    assert!(!done(back.hints, Hint::Minimap));
    let old: Settings = toml::from_str("[graphics]\nfov = 90\n").unwrap();
    assert_eq!(old.hints, 0);
}

/// There are at most four hints, each with its own bit.
#[test]
fn there_are_four_hints_with_distinct_bits() {
    let all = Hint::ALL.iter().fold(0, |bits, hint| bits | hint.bit());
    assert_eq!(Hint::ALL.len(), 4);
    assert_eq!(all.count_ones(), 4);
}
