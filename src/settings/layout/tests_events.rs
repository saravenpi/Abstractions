//! Tests of the labels inside an app: real key events teach them.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::{KeyLabels, plugin};

/// Returns the logical key of a key that types `text`.
fn types(text: &str) -> Key {
    Key::Character(text.into())
}

/// Sends one key event to `app` and runs a frame.
fn send(app: &mut App, code: KeyCode, logical: Key, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code: code,
        logical_key: logical,
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

/// In an app, real key events teach the labels: a press alone does, and a press with Shift or
/// AltGr held does not, so the label is what the key prints alone.
#[test]
fn key_events_teach_the_labels_unless_a_modifier_is_held() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::input::InputPlugin, plugin));
    let label = |app: &App, key| app.world().resource::<KeyLabels>().label(key);
    send(&mut app, KeyCode::KeyA, types("q"), ButtonState::Pressed);
    assert_eq!(label(&app, KeyCode::KeyA), "Q");
    assert_eq!(label(&app, KeyCode::KeyW), "Z");
    let held = [
        (KeyCode::ShiftLeft, Key::Shift, KeyCode::Comma, ".", ";"),
        (KeyCode::AltRight, Key::AltGraph, KeyCode::KeyE, "€", "E"),
    ];
    for (modifier, logical, key, typed, kept) in held {
        send(&mut app, modifier, logical.clone(), ButtonState::Pressed);
        send(&mut app, key, types(typed), ButtonState::Pressed);
        send(&mut app, modifier, logical, ButtonState::Released);
        assert_eq!(label(&app, key), kept, "{modifier:?} taught nothing");
    }
}
