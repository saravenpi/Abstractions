//! Unit tests for the rebindable actions and the conflicts between them.

use bevy::prelude::KeyCode;

use super::binds::{Bind, conflicted, conflicts};
use super::field::{Field, set_key};
use super::format::{display, key_name};
use super::hint::conflict_line;
use crate::settings::{Settings, ShipKeys};

/// The default bindings have no conflict, although walking and flying share keys.
#[test]
fn default_bindings_do_not_conflict() {
    let keys = Settings::default().controls.keys;
    assert_eq!(keys.forward, keys.ship.pitch_down);
    assert_eq!(conflicts(&keys), Vec::new());
    assert_eq!(conflict_line(&keys), "");
}

/// Two actions on foot with one key conflict, and so do two actions in the ship.
#[test]
fn a_shared_key_inside_one_group_is_a_conflict() {
    let mut s = Settings::default();
    set_key(&mut s, Bind::Jump, KeyCode::KeyW);
    let keys = s.controls.keys;
    assert_eq!(conflicts(&keys), vec![(Bind::Forward, Bind::Jump)]);
    assert!(conflicted(&keys, Bind::Forward) && conflicted(&keys, Bind::Jump));
    assert!(!conflicted(&keys, Bind::Thrust) && !conflicted(&keys, Bind::Back));
    assert_eq!(
        conflict_line(&keys),
        "W is bound to both \"Forward\" and \"Jump\""
    );
    set_key(&mut s, Bind::Engage, KeyCode::KeyJ);
    let found = conflicts(&s.controls.keys);
    assert_eq!(
        found,
        vec![(Bind::Forward, Bind::Jump), (Bind::Warp, Bind::Engage)]
    );
    assert!(conflict_line(&s.controls.keys).ends_with(", and 1 more"));
}

/// The board key is used on foot and in the ship, so it conflicts with both groups.
#[test]
fn the_board_key_conflicts_with_both_groups() {
    let mut s = Settings::default();
    set_key(&mut s, Bind::Board, KeyCode::Space);
    let found = conflicts(&s.controls.keys);
    assert_eq!(
        found,
        vec![(Bind::Jump, Bind::Board), (Bind::Up, Bind::Board)]
    );
    set_key(&mut s, Bind::Board, KeyCode::Tab);
    assert_eq!(
        conflicts(&s.controls.keys),
        vec![(Bind::Board, Bind::Computer)]
    );
}

/// Every action reads and writes its own key, and the 14 ship keys all have a row.
#[test]
fn every_action_has_its_own_key() {
    let all: Vec<Bind> = Bind::FOOT.into_iter().chain(Bind::SHIP).collect();
    let mut s = Settings::default();
    for (index, bind) in all.iter().enumerate() {
        let key = [KeyCode::F1, KeyCode::F2, KeyCode::F3][index % 3];
        set_key(&mut s, *bind, key);
        assert_eq!(bind.key(&s.controls.keys), key, "{bind:?}");
        assert_eq!(display(&s, Field::Key(*bind)), key_name(key));
        set_key(&mut s, *bind, KeyCode::F12);
    }
    let ship = s.controls.keys.ship;
    let twelve = ShipKeys {
        throttle_up: KeyCode::F12,
        ..ship
    };
    assert_eq!(ship, twelve);
    assert_eq!(toml::to_string(&ship).unwrap().matches("F12").count(), 14);
    assert_eq!(
        (s.controls.keys.computer, s.controls.keys.engage),
        (KeyCode::F12, KeyCode::F12)
    );
    assert_eq!(key_name(KeyCode::ShiftLeft), "Left Shift");
}

/// The map and waypoint keys are rows on foot and in the ship, and clash with keys of both. The
/// turn keys also turn the wingsuit, so a map key on Q clashes with them in both groups.
#[test]
fn the_map_and_waypoint_keys_conflict_in_both_groups() {
    assert!(Bind::FOOT.contains(&Bind::Map) && Bind::SHIP.contains(&Bind::Waypoint));
    let mut s = Settings::default();
    set_key(&mut s, Bind::Waypoint, KeyCode::KeyM);
    assert_eq!(
        conflicts(&s.controls.keys),
        vec![(Bind::Map, Bind::Waypoint), (Bind::Map, Bind::Waypoint)]
    );
    set_key(&mut s, Bind::Map, KeyCode::KeyQ);
    assert_eq!(
        conflicts(&s.controls.keys),
        vec![(Bind::YawLeft, Bind::Map), (Bind::YawLeft, Bind::Map)]
    );
}

/// The field guide key is a row on foot only, on K by default and in files written before it
/// existed, and it clashes with another key on foot but never with a key of the ship.
#[test]
fn the_field_guide_key_conflicts_on_foot_only() {
    assert!(Bind::FOOT.contains(&Bind::Guide) && !Bind::SHIP.contains(&Bind::Guide));
    let old = "[controls.keys]\nmap = \"KeyM\"\nwaypoint = \"KeyB\"\n";
    let keys = toml::from_str::<Settings>(old).unwrap().controls.keys;
    assert_eq!(keys.guide, KeyCode::KeyK);
    let mut s = Settings::default();
    set_key(&mut s, Bind::Guide, KeyCode::KeyM);
    assert_eq!(conflicts(&s.controls.keys), vec![(Bind::Map, Bind::Guide)]);
    set_key(&mut s, Bind::Guide, KeyCode::KeyZ);
    assert_eq!(conflicts(&s.controls.keys), Vec::new());
}
