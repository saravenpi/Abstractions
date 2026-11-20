//! Interaction look of buttons: their background and the text colours that follow.

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

/// Marks a button as the chosen one of its group: the open tab, the destination row, the armed
/// choice. It changes the look only, never the keyboard focus.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selected(pub bool);

/// Kind of a text inside a button. There is no hue: kinds differ by brightness only, and every
/// text of the current row turns black on its white block.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// White.
    Plain,
    /// White, for the main action of a group. It is told apart by its place, not by a colour.
    Accent,
    /// Dimmed white.
    Muted,
    /// White, for a destructive action. Its wording and its confirmation mark it, not a colour.
    Danger,
    /// Hidden until its row is the current one.
    Marker,
}

/// Returns the text color of a tone in a given look.
pub fn tone_color(tone: Tone, look: Look) -> Color {
    match (tone, look) {
        (Tone::Marker, Look::Rest) => Color::NONE,
        (Tone::Muted, Look::Rest) => theme::TEXT_2,
        (_, Look::Rest) => theme::TEXT,
        (_, Look::Active | Look::Pressed) => theme::INK,
    }
}

/// Returns the text color of a tone inside a button: a selected button shows its dimmed texts
/// at full brightness.
pub fn shade(tone: Tone, look: Look, selected: bool) -> Color {
    let shown = match (selected, tone) {
        (true, Tone::Muted) => Tone::Plain,
        _ => tone,
    };
    tone_color(shown, look)
}

/// Returns the background of a button: a white block under the current one, a faint block
/// under a selected one, nothing otherwise.
pub fn surface(look: Look, selected: bool) -> Color {
    match (look, selected) {
        (Look::Pressed, _) => theme::BLOCK_PRESSED,
        (Look::Active, _) => theme::BLOCK,
        (Look::Rest, true) => theme::SELECT,
        (Look::Rest, false) => Color::NONE,
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

/// Everything the look of one button depends on, and what shows it.
#[derive(QueryData)]
#[query_data(mutable)]
pub(super) struct ButtonState {
    entity: Entity,
    look: &'static mut Look,
    background: &'static mut BackgroundColor,
    hovered: &'static Hovered,
    selected: Option<&'static Selected>,
    pressed: Has<Pressed>,
    focusable: Has<TabIndex>,
}

/// Derives each button's look from focus, hover and press state, then its background.
///
/// A focusable button is the current one while it holds the focus. A pointer-only button is
/// the current one while hovered. The current button is inverted: black on a white block.
pub(super) fn update_looks(focus: Res<InputFocus>, mut buttons: Query<ButtonState>) {
    for mut button in &mut buttons {
        let focused = button.focusable && focus.get() == Some(button.entity);
        let active = focused || (!button.focusable && button.hovered.get());
        let wanted = match (button.pressed, active) {
            (true, _) => Look::Pressed,
            (false, true) => Look::Active,
            _ => Look::Rest,
        };
        button.look.set_if_neq(wanted);
        let selected = button.selected.is_some_and(|selected| selected.0);
        button
            .background
            .set_if_neq(BackgroundColor(surface(wanted, selected)));
    }
}

/// What a text colour depends on: a changed look, selection or tone anywhere.
type Repaint = Or<(Changed<Look>, Changed<Selected>, Changed<Tone>)>;

/// Colors every toned text by the look of its nearest button ancestor. Does nothing on a frame
/// where no look, selection or tone changed.
pub(super) fn paint_texts(
    changed: Query<(), Repaint>,
    looks: Query<(&Look, Option<&Selected>)>,
    parents: Query<&ChildOf>,
    mut texts: Query<(Entity, &Tone, &mut TextColor)>,
) {
    if changed.is_empty() {
        return;
    }
    for (entity, tone, mut color) in &mut texts {
        let (look, selected) = parents
            .iter_ancestors(entity)
            .find_map(|ancestor| looks.get(ancestor).ok())
            .map_or((Look::Rest, false), |(look, selected)| {
                (*look, selected.is_some_and(|selected| selected.0))
            });
        color.set_if_neq(TextColor(shade(*tone, look, selected)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A marker is invisible at rest and shows on the white block of the current row, where
    /// every text is black.
    #[test]
    fn marker_shows_only_when_active() {
        assert_eq!(tone_color(Tone::Marker, Look::Rest), Color::NONE);
        assert_eq!(tone_color(Tone::Marker, Look::Active), theme::INK);
        assert_eq!(tone_color(Tone::Muted, Look::Active), theme::INK);
    }

    /// The current row is a white block, a selected row shows without the focus and brightens
    /// its dimmed texts, and a resting row has no background.
    #[test]
    fn the_current_row_is_inverted() {
        assert_eq!(surface(Look::Active, false), theme::BLOCK);
        assert_eq!(surface(Look::Active, true), theme::BLOCK);
        assert_eq!(surface(Look::Rest, true), theme::SELECT);
        assert_eq!(surface(Look::Rest, false), Color::NONE);
        assert_eq!(shade(Tone::Muted, Look::Rest, false), theme::TEXT_2);
        assert_eq!(shade(Tone::Muted, Look::Rest, true), theme::TEXT);
        assert_eq!(shade(Tone::Danger, Look::Rest, false), theme::TEXT);
        assert_eq!(shade(Tone::Plain, Look::Pressed, true), theme::INK);
    }
}
