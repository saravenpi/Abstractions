//! Typing in the panel: letters narrow the list, Backspace edits the filter, Escape clears it
//! and a key changes the order of the rows.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use super::Computer;
use super::board::Board;
use super::route::Destination;
use super::sift::type_into;
use crate::settings::{Persistent, Settings};

/// The key that changes the order of the rows.
pub(super) const SORT_KEY: KeyCode = KeyCode::F2;

/// Applies one pressed key to the filter and returns true when the filter changed.
fn edit(filter: &mut String, key: &Key) -> bool {
    match key {
        Key::Backspace => filter.pop().is_some(),
        Key::Escape if !filter.is_empty() => {
            filter.clear();
            true
        }
        Key::Character(text) => type_into(filter, text),
        _ => false,
    }
}

/// Edits the filter with the keys pressed this frame and changes the order on its key. The
/// list is then built again from its first row.
///
/// The engage key engages while a course is set and no filter is typed, so it types only in
/// the other cases. Keys pressed before the panel opened, such as a held strafe key, are
/// dropped on the frame it opens.
pub(super) fn type_filter(
    mut typed: MessageReader<KeyboardInput>,
    (keys, opened): (Res<ButtonInput<KeyCode>>, Res<State<Computer>>),
    settings: Res<Persistent<Settings>>,
    course: Res<Destination>,
    mut board: ResMut<Board>,
) {
    if opened.is_changed() {
        typed.clear();
        return;
    }
    if typed.is_empty() && !keys.just_pressed(SORT_KEY) {
        return;
    }
    let mut filter = board.filter.clone();
    let engage = settings.controls.keys.engage;
    let mut changed = keys.just_pressed(SORT_KEY);
    for key in typed.read().filter(|key| key.state == ButtonState::Pressed) {
        let engaging = key.key_code == engage && course.0.is_some() && filter.is_empty();
        changed |= !engaging && edit(&mut filter, &key.logical_key);
    }
    if !changed {
        return;
    }
    if keys.just_pressed(SORT_KEY) {
        board.sort = board.sort.next();
    }
    (board.filter, board.stale) = (filter, true);
    (board.cursor, board.top) = (0, 0);
}
