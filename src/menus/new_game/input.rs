//! Typing and showing the name and seed fields.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::{CreateCtx, Kind, NewWorld};
use crate::menus::slots::Slot;
use crate::menus::{name, seed};
use crate::ui::theme;

/// Returns the field kind that holds the focus, if any.
fn focused_kind(focus: &InputFocus, kinds: &Query<&Kind>) -> Option<Kind> {
    focus
        .get()
        .and_then(|entity| kinds.get(entity).ok())
        .copied()
}

/// Returns the text and color a field shows.
fn shown(kind: Kind, value: &str, focused: bool) -> (String, Color) {
    match (focused, value.is_empty(), kind) {
        (true, _, _) => (name::with_caret(value), theme::INK),
        (false, false, _) => (value.to_string(), theme::TEXT),
        (false, true, Kind::Name) => ("name your world".to_string(), theme::TEXT_2),
        (false, true, Kind::Seed) => ("Empty means random".to_string(), theme::TEXT_2),
    }
}

/// Types into the focused field.
pub(super) fn type_text(
    mut keys: MessageReader<KeyboardInput>,
    focus: Res<InputFocus>,
    kinds: Query<&Kind>,
    mut input: ResMut<NewWorld>,
) {
    let kind = focused_kind(&focus, &kinds);
    for key in keys.read().filter(|key| key.state == ButtonState::Pressed) {
        let backspace = key.logical_key == Key::Backspace;
        let typed = key.text.as_deref().unwrap_or("");
        match kind {
            Some(Kind::Name) => name::edit(&mut input.name, backspace, typed),
            Some(Kind::Seed) => seed::edit(&mut input.seed, backspace, typed),
            None => {}
        }
    }
}

/// Moves focus to the other field on Tab.
pub(super) fn switch_field(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    fields: Query<(Entity, &Kind), With<Slot>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }
    let current = focus.get().and_then(|entity| fields.get(entity).ok());
    let wanted = match current {
        Some((_, Kind::Name)) => Kind::Seed,
        _ => Kind::Name,
    };
    if let Some((entity, _)) = fields.iter().find(|(_, kind)| **kind == wanted) {
        focus.set(entity, FocusCause::Navigated);
    }
}

/// Creates the world when Enter is pressed on a field.
pub(super) fn create_on_enter(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    kinds: Query<&Kind>,
    mut ctx: CreateCtx,
) {
    let enter = keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]);
    if enter && focused_kind(&focus, &kinds).is_some() {
        ctx.create();
    }
}

/// Shows each field's text, with a caret on the focused one.
pub(super) fn show_fields(
    input: Res<NewWorld>,
    focus: Res<InputFocus>,
    rows: Query<(Entity, &Kind), With<Slot>>,
    mut texts: Query<(&Kind, &mut Text, &mut TextColor), Without<Slot>>,
) {
    let active = focus
        .get()
        .and_then(|entity| rows.get(entity).ok())
        .map(|(_, kind)| *kind);
    for (kind, mut text, mut tint) in &mut texts {
        let value = match kind {
            Kind::Name => &input.name,
            Kind::Seed => &input.seed,
        };
        let (content, color) = shown(*kind, value, active == Some(*kind));
        text.set_if_neq(Text(content));
        tint.set_if_neq(TextColor(color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A focused field shows a caret and an empty idle field shows its hint.
    #[test]
    fn shows_caret_and_hint() {
        assert_eq!(shown(Kind::Name, "Isle", true).0, "Isle_");
        assert_eq!(shown(Kind::Seed, "", false).1, theme::TEXT_2);
        assert_eq!(shown(Kind::Seed, "12", false).0, "12");
    }
}
