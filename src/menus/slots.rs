//! Keyboard order for menus whose letters are typed or whose lists scroll.

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::Menu;

/// Position of a focusable element in the keyboard order of its menu.
#[derive(Component, Debug, Clone, Copy)]
pub struct Slot(pub usize);

/// Returns the slot after moving `dir` steps from `current` among `len`, wrapping around.
fn next_slot(len: usize, current: Option<usize>, dir: i32) -> usize {
    match (current, dir > 0) {
        (None, true) => 0,
        (None, false) => len - 1,
        (Some(at), true) => (at + 1) % len,
        (Some(at), false) => (at + len - 1) % len,
    }
}

/// Run condition: the open menu moves focus by slots instead of by screen position.
pub fn uses_slots(menu: Res<State<Menu>>) -> bool {
    matches!(menu.get(), Menu::Worlds | Menu::NewGame)
}

/// Moves focus between slots with the arrow keys, and W and S where letters are not typed.
pub fn move_slots(
    keys: Res<ButtonInput<KeyCode>>,
    menu: Res<State<Menu>>,
    mut focus: ResMut<InputFocus>,
    slots: Query<(Entity, &Slot)>,
) {
    let letters = *menu.get() != Menu::NewGame;
    let down =
        keys.just_pressed(KeyCode::ArrowDown) || (letters && keys.just_pressed(KeyCode::KeyS));
    let up = keys.just_pressed(KeyCode::ArrowUp) || (letters && keys.just_pressed(KeyCode::KeyW));
    if down == up {
        return;
    }
    let mut ordered: Vec<(usize, Entity)> = slots
        .iter()
        .map(|(entity, slot)| (slot.0, entity))
        .collect();
    if ordered.is_empty() {
        return;
    }
    ordered.sort();
    let current = ordered
        .iter()
        .position(|(_, entity)| Some(*entity) == focus.get());
    let target = ordered[next_slot(ordered.len(), current, if down { 1 } else { -1 })].1;
    focus.set(target, FocusCause::Navigated);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Moves wrap at both ends and start at the matching edge.
    #[test]
    fn wraps_around() {
        assert_eq!(next_slot(3, Some(2), 1), 0);
        assert_eq!(next_slot(3, Some(0), -1), 2);
        assert_eq!(next_slot(3, None, 1), 0);
        assert_eq!(next_slot(3, None, -1), 2);
    }
}
