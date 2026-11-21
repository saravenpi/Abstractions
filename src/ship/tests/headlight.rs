//! The headlight switch: its key turns it over only for a seated pilot whose keyboard is free,
//! and the save file keeps it.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::{Ship, ShipState};
use crate::ship::headlight::switched;
use crate::space::Frame;
use crate::universe::BodyId;

/// The key switches the headlight on and off again; without the key, on foot or while the
/// board computer holds the keyboard it stays as it is.
#[test]
fn the_key_switches_the_headlight() {
    assert!(switched(false, true, (true, false)));
    assert!(!switched(true, true, (true, false)));
    for held in [false, true] {
        assert_eq!(switched(held, false, (true, false)), held);
        assert_eq!(switched(held, true, (false, false)), held);
        assert_eq!(switched(held, true, (true, true)), held);
    }
}

/// A ship is built with its headlight off, a saved ship comes back with the headlight it had,
/// and a save written before the headlight had a switch reads as off.
#[test]
fn the_headlight_survives_a_save() {
    let frame = Frame::Surface(BodyId::default());
    let mut ship = Ship::at_rest(frame, DVec3::Y * 6000.0, DQuat::IDENTITY);
    assert!(!ship.headlight);
    ship.headlight = true;
    let text = toml::to_string(&ship.state()).unwrap();
    let state: ShipState = toml::from_str(&text).unwrap();
    assert!(Ship::from_state(&state).unwrap().headlight);
    let old: String = text
        .lines()
        .filter(|line| !line.starts_with("headlight"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(old.len() < text.len());
    let state: ShipState = toml::from_str(&old).unwrap();
    assert!(!Ship::from_state(&state).unwrap().headlight);
}
