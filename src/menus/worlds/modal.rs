//! Keyboard handling of the worlds menu while a name is typed or a delete is confirmed.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::Mode;
use super::act::{Act, Ctx};
use crate::menus::name;

/// Returns what Enter does while a delete is confirmed: it deletes only once that choice is armed.
pub(super) fn enter_choice(armed: bool) -> Act {
    match armed {
        true => Act::Confirm,
        false => Act::Cancel,
    }
}

/// Types into the name field. Enter applies the name and Escape drops it. The key that opened
/// the field is not typed.
pub(super) fn rename_keys(mut typed: MessageReader<KeyboardInput>, mut ctx: Ctx) {
    if ctx.mode.is_changed() || !matches!(*ctx.mode, Mode::Rename(_)) {
        typed.clear();
        return;
    }
    for key in typed.read().filter(|key| key.state == ButtonState::Pressed) {
        match key.logical_key {
            Key::Enter => return ctx.run(Act::Confirm),
            Key::Escape => return ctx.run(Act::Cancel),
            _ => {}
        }
        if let Mode::Rename(text) = &mut *ctx.mode {
            let letters = key.text.as_deref().unwrap_or("");
            name::edit(text, key.logical_key == Key::Backspace, letters);
        }
    }
}

/// Chooses between keeping and deleting with Left, Right and Tab. Enter applies the choice,
/// which starts on keeping, and Escape keeps the world.
pub(super) fn delete_keys(keys: Res<ButtonInput<KeyCode>>, mut ctx: Ctx) {
    let Mode::Delete { armed } = *ctx.mode else {
        return;
    };
    if ctx.mode.is_changed() {
        return;
    }
    let switch = [KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::Tab];
    if keys.any_just_pressed(switch) {
        *ctx.mode = Mode::Delete { armed: !armed };
    } else if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        ctx.run(enter_choice(armed));
    } else if keys.just_pressed(KeyCode::Escape) {
        ctx.run(Act::Cancel);
    }
}
