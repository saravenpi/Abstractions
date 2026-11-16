//! Keyboard focus movement between menu buttons.

use bevy::input_focus::{FocusCause, InputFocus, tab_navigation::TabIndex};
use bevy::prelude::*;
use bevy::ui::{ComputedNode, UiGlobalTransform};

/// Returns the index to focus after moving `dir` steps from `current`, wrapping around.
fn next_index(len: usize, current: Option<usize>, dir: i32) -> usize {
    match (current, dir > 0) {
        (None, true) => 0,
        (None, false) => len - 1,
        (Some(at), true) => (at + 1) % len,
        (Some(at), false) => (at + len - 1) % len,
    }
}

/// Moves focus between visible focusable buttons with Up/Down or W/S.
pub fn move_focus(
    keys: Res<ButtonInput<KeyCode>>,
    mut focus: ResMut<InputFocus>,
    buttons: Query<(Entity, &TabIndex, &ComputedNode, &UiGlobalTransform)>,
) {
    let down = keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]);
    let up = keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]);
    if down == up {
        return;
    }
    let mut ordered: Vec<(i32, i32, Entity)> = buttons
        .iter()
        .filter(|(_, _, node, _)| node.size().x > 0.0)
        .map(|(entity, index, _, at)| (index.0, at.translation.y as i32, entity))
        .collect();
    if ordered.is_empty() {
        return;
    }
    ordered.sort();
    let current = ordered
        .iter()
        .position(|(_, _, entity)| Some(*entity) == focus.get());
    let target = ordered[next_index(ordered.len(), current, if down { 1 } else { -1 })].2;
    focus.set(target, FocusCause::Navigated);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Focus wraps at both ends and starts at the matching edge.
    #[test]
    fn wraps_around() {
        assert_eq!(next_index(3, Some(2), 1), 0);
        assert_eq!(next_index(3, Some(0), -1), 2);
        assert_eq!(next_index(3, None, 1), 0);
        assert_eq!(next_index(3, None, -1), 2);
    }
}
