//! Tests of the default key bindings: they are stored as physical keys, and every one of them is
//! a key that the smallest common keyboards have.

use bevy::prelude::KeyCode;

use super::{Keymap, Settings};

/// The code names of the keys both an ANSI and an ISO keyboard of 60 percent have without a
/// function layer: the letters, the number row, the signs beside them and the keys around the
/// letters. Such a keyboard has no function row, no arrows, no pad, no Delete, and Esc stands
/// where the backquote is; the extra ISO key is missing on ANSI.
const SIXTY: [&str; 17] = [
    "Minus",
    "Equal",
    "BracketLeft",
    "BracketRight",
    "Semicolon",
    "Quote",
    "Comma",
    "Period",
    "Slash",
    "Space",
    "Tab",
    "Enter",
    "Backspace",
    "Escape",
    "ShiftLeft",
    "ControlLeft",
    "AltLeft",
];

/// Returns true when the key called `name` is on a keyboard of 60 percent.
fn on_sixty(name: &str) -> bool {
    let letter = name.strip_prefix("Key").is_some_and(|rest| rest.len() == 1);
    let digit = name
        .strip_prefix("Digit")
        .is_some_and(|rest| rest.len() == 1);
    letter || digit || SIXTY.contains(&name)
}

/// Returns the code name of every key bound in `keys`, with the name of its action.
fn bound(keys: &Keymap) -> Vec<(String, String)> {
    let saved = toml::to_string(keys).expect("the bindings are saved");
    let table: toml::Table = saved.parse().expect("the bindings are read back");
    let ship = table["ship"].as_table().expect("the ship keys").clone();
    let all = table.into_iter().chain(ship);
    all.filter_map(|(action, key)| Some((action, key.as_str()?.to_string())))
        .collect()
}

/// Every default binding, on foot and in the ship, is a key of a 60 percent keyboard, ANSI or
/// ISO: nobody has to rebind before they can play.
#[test]
fn every_default_key_is_on_a_sixty_percent_keyboard() {
    let keys = bound(&Keymap::default());
    assert_eq!(keys.len(), 17 + 14, "every action on foot and in the ship");
    for (action, key) in keys {
        assert!(on_sixty(&key), "{action} is on {key}");
    }
    assert!(!on_sixty("IntlBackslash") && !on_sixty("Numpad1") && !on_sixty("F8"));
    assert!(!on_sixty("Backquote") && !on_sixty("Delete") && !on_sixty("ArrowUp"));
}

/// A binding is saved under the name of its physical key, which no layout changes, and a file
/// written by an older version, with only some of the keys, still loads.
#[test]
fn bindings_are_saved_as_physical_keys() {
    let mut settings = Settings::default();
    settings.controls.keys.forward = KeyCode::KeyY;
    let saved = toml::to_string(&settings).expect("the settings are saved");
    assert!(saved.contains("forward = \"KeyY\""), "{saved}");
    assert!(saved.contains("jump = \"Space\""), "{saved}");
    let back: Settings = toml::from_str(&saved).expect("the settings are read back");
    assert_eq!(back.controls.keys, settings.controls.keys);
    let old = "[controls.keys]\nforward = \"KeyZ\"\nleft = \"KeyQ\"\n";
    let keys = toml::from_str::<Settings>(old).unwrap().controls.keys;
    assert_eq!((keys.forward, keys.left), (KeyCode::KeyZ, KeyCode::KeyQ));
    assert_eq!(keys.back, Keymap::default().back);
}
