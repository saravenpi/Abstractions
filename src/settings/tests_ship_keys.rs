//! The keys of the ship: every action has a key of its own by default, and a settings file
//! written before the headlight had a key still loads.

use std::collections::HashSet;

use bevy::prelude::KeyCode;

use super::ShipKeys;

/// The fifteen default keys of the ship are all different.
#[test]
fn every_default_ship_key_is_distinct() {
    let keys = ShipKeys::default();
    let text = toml::to_string(&keys).unwrap();
    let bound: Vec<&str> = text
        .lines()
        .filter_map(|line| line.split('=').nth(1))
        .collect();
    let distinct: HashSet<&str> = bound.iter().copied().collect();
    assert_eq!((bound.len(), distinct.len()), (15, 15), "{text}");
    assert_eq!(keys.headlight, KeyCode::KeyL);
}

/// A settings file without the headlight key loads with the default one, and keeps its own keys.
#[test]
fn old_ship_keys_still_load() {
    let old: ShipKeys = toml::from_str("warp = \"KeyK\"").unwrap();
    assert_eq!((old.warp, old.headlight), (KeyCode::KeyK, KeyCode::KeyL));
}
