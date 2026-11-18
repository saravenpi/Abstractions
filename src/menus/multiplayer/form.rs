//! The form of the multiplayer menu: the player name, direct connect, and adding or editing a
//! server. It takes keys like the rename of the worlds menu.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::act::{Act, Ctx};
use super::ops::ADDRESS_CHARS;
use super::{Field, Form, FormKind, Mode};
use crate::menus::name::{self, MAX_CHARS};

impl Form {
    /// Returns a form of `kind` holding `name` and `address`, typing into its first field.
    pub(super) fn new(kind: FormKind, name: String, address: String) -> Self {
        let field = match kind {
            FormKind::Direct => Field::Address,
            _ => Field::Name,
        };
        Self {
            kind,
            name,
            address,
            field,
        }
    }

    /// Moves to the other field, when the form has two.
    pub(super) fn switch(&mut self) {
        if matches!(self.kind, FormKind::Add | FormKind::Edit(_)) {
            self.field = match self.field {
                Field::Name => Field::Address,
                Field::Address => Field::Name,
            };
        }
    }

    /// Types `letters` into the current field, after one backspace when `backspace` is set.
    pub(super) fn type_text(&mut self, backspace: bool, letters: &str) {
        match self.field {
            Field::Name => name::edit_to(&mut self.name, backspace, letters, MAX_CHARS),
            Field::Address => name::edit_to(&mut self.address, backspace, letters, ADDRESS_CHARS),
        }
    }
}

/// Types into the form. Enter applies it, Escape drops it and Tab moves to the other field. The
/// key that opened the form is not typed.
pub(super) fn type_keys(mut typed: MessageReader<KeyboardInput>, mut ctx: Ctx) {
    if ctx.mode.is_changed() || !matches!(*ctx.mode, Mode::Form(_)) {
        typed.clear();
        return;
    }
    for key in typed.read().filter(|key| key.state == ButtonState::Pressed) {
        match key.logical_key {
            Key::Enter => return ctx.run(Act::Confirm),
            Key::Escape => return ctx.run(Act::Cancel),
            _ => {}
        }
        if let Mode::Form(form) = &mut *ctx.mode {
            match key.logical_key {
                Key::Tab => form.switch(),
                _ => {
                    let letters = key.text.as_deref().unwrap_or("");
                    form.type_text(key.logical_key == Key::Backspace, letters);
                }
            }
        }
    }
}
