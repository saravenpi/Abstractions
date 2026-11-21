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

/// Marks a button that rests on a faint fill, such as a text field: it has no outline, so the fill
/// shows where it is.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Filled;

/// Marks a button as the chosen one of its group: the open tab, the destination row, the armed
/// choice. It changes the look only, never the keyboard focus.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selected(pub bool);

/// Kind of a text inside a button. Kinds differ by brightness, and by the accent for the main
/// action. The row under the pointer is tinted, its texts do not change.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// White.
    Plain,
    /// The accent colour, for the main action of a group.
    Accent,
    /// Dimmed white.
    Muted,
    /// White, for a destructive action. Its wording and its confirmation mark it.
    Danger,
    /// Hidden until its row is the current one.
    Marker,
    /// The name of a key: the secondary text colour, whatever the look of its button.
    Key,
}

/// Returns the text color of a tone in a given look.
pub fn tone_color(tone: Tone, look: Look) -> Color {
    match (tone, look) {
        (Tone::Marker, Look::Rest) => Color::NONE,
        (Tone::Marker, _) | (Tone::Accent, _) => theme::ACCENT,
        (Tone::Muted, Look::Rest) | (Tone::Key, _) => theme::TEXT_2,
        _ => theme::TEXT,
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

/// Returns the background of a button: a tint under the current one, the selection tint under
/// a selected one, nothing otherwise. Never a white block.
pub fn surface(look: Look, selected: bool) -> Color {
    match (look, selected) {
        (Look::Pressed, _) | (_, true) => theme::SELECT,
        (Look::Active, false) => theme::HOVER,
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
    filled: Has<Filled>,
}

/// Derives each button's look from focus, hover and press state, then its background.
///
/// A focusable button is the current one while it holds the focus. A pointer-only button is
/// the current one while hovered. The current button is tinted with `HOVER`.
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
        let rests = button.filled && wanted == Look::Rest && !selected;
        let fill = match rests {
            true => theme::HOVER,
            false => surface(wanted, selected || (button.filled && wanted != Look::Rest)),
        };
        button.background.set_if_neq(BackgroundColor(fill));
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

    /// A marker is invisible at rest and shows in the accent on the current row.
    #[test]
    fn marker_shows_only_when_active() {
        assert_eq!(tone_color(Tone::Marker, Look::Rest), Color::NONE);
        assert_eq!(tone_color(Tone::Marker, Look::Active), theme::ACCENT);
        assert_eq!(tone_color(Tone::Muted, Look::Active), theme::TEXT);
    }

    /// The current row is tinted, never a white block, a selected row keeps its tint and
    /// brightens its dimmed texts, and a resting row has no background.
    #[test]
    fn the_current_row_is_tinted() {
        assert_eq!(surface(Look::Active, false), theme::HOVER);
        assert_eq!(surface(Look::Active, true), theme::SELECT);
        assert_eq!(surface(Look::Rest, true), theme::SELECT);
        assert_eq!(surface(Look::Rest, false), Color::NONE);
        assert_eq!(shade(Tone::Muted, Look::Rest, false), theme::TEXT_2);
        assert_eq!(shade(Tone::Muted, Look::Rest, true), theme::TEXT);
        assert_eq!(shade(Tone::Danger, Look::Rest, false), theme::TEXT);
        assert_eq!(shade(Tone::Plain, Look::Pressed, true), theme::TEXT);
    }
}
