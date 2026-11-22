//! Keyboard layouts and the body: the keys of walking and of the hotbar are physical keys, so
//! the same hand position walks and picks whatever the layout types there, and the labels name
//! those keys as the layout prints them.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardFocusLost, KeyboardInput, keyboard_input_system};
use bevy::prelude::*;

use super::eyes::gliding;
use crate::craft::wield;
use crate::player::keys::send;
use crate::settings::{KeyLabels, Keymap};
use crate::sim::{Intent, IntentOf};
use crate::view::ViewPrefs;

/// The physical keys pressed: where W, A and 1 are on QWERTY.
const KEYS: [KeyCode; 3] = [KeyCode::KeyW, KeyCode::KeyA, KeyCode::Digit1];

/// Three layouts: what each types on the keys of [`KEYS`], and what it prints on the first two.
const LAYOUTS: [(&str, [&str; 3], [&str; 2]); 3] = [
    ("AZERTY", ["z", "q", "&"], ["Z", "Q"]),
    ("QWERTZ", ["w", "a", "1"], ["W", "A"]),
    ("Dvorak", [",", "a", "1"], [",", "A"]),
];

/// Returns a world whose keyboard got a real event for each key of [`KEYS`], typing `typed`,
/// read into the key state as the input plugin does.
fn pressed(typed: [&str; 3]) -> World {
    let mut world = gliding();
    world.init_resource::<ViewPrefs>();
    world.init_resource::<ButtonInput<Key>>();
    world.init_resource::<Messages<IntentOf>>();
    world.init_resource::<Messages<KeyboardInput>>();
    world.init_resource::<Messages<KeyboardFocusLost>>();
    for (key_code, text) in KEYS.into_iter().zip(typed) {
        world.write_message(KeyboardInput {
            key_code,
            logical_key: Key::Character(text.into()),
            state: ButtonState::Pressed,
            text: Some(text.into()),
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }
    world
        .run_system_once(keyboard_input_system)
        .expect("the keys are read");
    world
}

/// Returns the labels after the keys of [`KEYS`] typed `typed`.
fn labels(typed: [&str; 3]) -> KeyLabels {
    let mut labels = KeyLabels::default();
    for (key, text) in KEYS.into_iter().zip(typed) {
        labels.learn(key, &Key::Character(text.into()));
    }
    labels
}

/// On AZERTY, QWERTZ and Dvorak the keys where W and A are walk forward and left, the key
/// where 1 is picks the first hotbar slot, and the labels show what the layout prints there:
/// Z and Q on AZERTY, with the digit 1 for the slot although the key types an ampersand.
#[test]
fn the_same_physical_keys_walk_and_pick_on_every_layout() {
    for (name, typed, printed) in LAYOUTS {
        let mut world = pressed(typed);
        world.run_system_once(send).expect("the intents are sent");
        let mut out = world.resource_mut::<Messages<IntentOf>>();
        let sent: Vec<Intent> = out.drain().map(|message| message.intent).collect();
        let walk = Intent::Move {
            forward: 1.0,
            strafe: -1.0,
        };
        assert!(sent.contains(&walk), "{name}: {sent:?}");
        let keys = world.resource::<ButtonInput<KeyCode>>();
        assert_eq!(wield::pressed(keys), Some(0), "{name}");
        let (labels, map) = (labels(typed), Keymap::default());
        let shown = [labels.label(map.forward), labels.label(map.left)];
        assert_eq!(shown, printed.map(String::from), "{name}");
        assert_eq!(labels.label(wield::KEYS[0]), "1", "{name}");
    }
}
