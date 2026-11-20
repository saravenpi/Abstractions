//! Naming a pin: the letters typed go into its name, Enter keeps it and Escape keeps the name
//! it had.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::Places;
use crate::computer::Typing;

/// Characters a name keeps at most.
pub const NAME_CHARS: usize = 32;

/// Applies one pressed key to a name and returns true when the name is done: Enter keeps it,
/// Escape gives it back as it was, and `before` is that name.
pub fn edit(name: &mut String, key: &Key, before: &str) -> bool {
    match key {
        Key::Enter => true,
        Key::Escape => {
            *name = before.to_string();
            true
        }
        Key::Backspace => {
            name.pop();
            false
        }
        Key::Character(text) if name.chars().count() < NAME_CHARS => {
            name.extend(text.chars().filter(|c| !c.is_control()));
            false
        }
        _ => false,
    }
}

/// Types into the name of the pin being renamed, and stores it when it is done.
pub(in crate::map) fn name_keys(
    mut typed: MessageReader<KeyboardInput>,
    mut typing: ResMut<Typing>,
    mut places: ResMut<Places>,
) {
    let Typing::Name(index, name) = &mut *typing else {
        typed.clear();
        return;
    };
    let index = *index;
    let before = places
        .pins
        .get(index)
        .map_or(String::new(), |pin| pin.name.clone());
    let mut pressed = typed.read().filter(|key| key.state == ButtonState::Pressed);
    let done = pressed.any(|key| edit(name, &key.logical_key, &before));
    if done {
        let name = name.clone();
        places.rename(index, &name);
        *typing = Typing::Off;
    }
}
