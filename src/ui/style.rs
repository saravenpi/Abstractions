//! Interaction look of buttons and the text colors that follow it.

use bevy::ecs::query::QueryData;
use bevy::input_focus::{FocusCause, InputFocus, tab_navigation::TabIndex};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;

use super::theme;

/// How a button currently looks.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Look {
    /// Nothing happens to the button.
    #[default]
    Rest,
    /// Focused or hovered.
    Active,
    /// Held down.
    Pressed,
}

/// Marks a button that slides right while it is active.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Shift;

/// Base color family of a text inside a button.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Off-white.
    Plain,
    /// Sand accent.
    Accent,
    /// Muted grey that brightens when active.
    Muted,
    /// Hidden until active, then accent.
    Marker,
}

/// Returns the text color of a tone in a given look.
pub fn tone_color(tone: Tone, look: Look) -> Color {
    match (tone, look) {
        (Tone::Plain, Look::Rest) => theme::ITEM,
        (Tone::Plain, Look::Active) => theme::TEXT,
        (Tone::Plain, Look::Pressed) => theme::ITEM_PRESSED,
        (Tone::Accent, Look::Rest) => theme::ACCENT_REST,
        (Tone::Accent, Look::Active) => theme::ACCENT,
        (Tone::Accent, Look::Pressed) => theme::ACCENT_PRESSED,
        (Tone::Muted, Look::Active) => theme::TEXT,
        (Tone::Muted, _) => theme::MUTED,
        (Tone::Marker, Look::Rest) => Color::NONE,
        (Tone::Marker, Look::Active) => theme::ACCENT,
        (Tone::Marker, Look::Pressed) => theme::ACCENT_PRESSED,
    }
}

/// Buttons whose hover state changed and that can hold focus.
type NewlyHovered = (Changed<Hovered>, With<TabIndex>);

/// Moves input focus to the focusable button the pointer enters.
pub(super) fn focus_on_hover(
    buttons: Query<(Entity, &Hovered), NewlyHovered>,
    mut focus: ResMut<InputFocus>,
) {
    for (entity, hovered) in &buttons {
        if hovered.get() && focus.get() != Some(entity) {
            focus.set(entity, FocusCause::Navigated);
        }
    }
}

/// Everything the look of one button depends on.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct ButtonState {
    entity: Entity,
    look: &'static mut Look,
    node: &'static mut Node,
    hovered: &'static Hovered,
    pressed: Has<Pressed>,
    focusable: Has<TabIndex>,
    shifts: Has<Shift>,
}

/// Returns the look of a button from its press and active state.
fn look_of(pressed: bool, active: bool) -> Look {
    match (pressed, active) {
        (true, _) => Look::Pressed,
        (false, true) => Look::Active,
        _ => Look::Rest,
    }
}

/// Derives each button's look from focus, hover and press state, and shifts focused items.
pub(super) fn update_looks(focus: Res<InputFocus>, mut buttons: Query<ButtonState>) {
    for mut button in &mut buttons {
        let active = if button.focusable {
            focus.get() == Some(button.entity)
        } else {
            button.hovered.get()
        };
        let wanted = look_of(button.pressed, active);
        button.look.set_if_neq(wanted);
        let shift = if button.shifts && wanted != Look::Rest {
            px(theme::SHIFT)
        } else {
            px(0)
        };
        if button.node.left != shift {
            button.node.left = shift;
        }
    }
}

/// Colors every toned text by the look of its nearest button ancestor.
pub(super) fn paint_texts(
    looks: Query<&Look>,
    parents: Query<&ChildOf>,
    mut texts: Query<(Entity, &Tone, &mut TextColor)>,
) {
    for (entity, tone, mut color) in &mut texts {
        let look = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| looks.get(ancestor).ok())
            .copied()
            .unwrap_or_default();
        color.set_if_neq(TextColor(tone_color(*tone, look)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A marker is invisible at rest and accent when active.
    #[test]
    fn marker_shows_only_when_active() {
        assert_eq!(tone_color(Tone::Marker, Look::Rest), Color::NONE);
        assert_eq!(tone_color(Tone::Marker, Look::Active), theme::ACCENT);
        assert_eq!(tone_color(Tone::Muted, Look::Active), theme::TEXT);
    }
}
