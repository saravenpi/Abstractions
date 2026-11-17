//! Keyboard and wheel handling of the worlds menu while moving through the list.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::act::{Act, Ctx};
use super::rows::Row;
use super::{VISIBLE_ROWS, Worlds};
use crate::menus::{Menu, Trail};
use crate::ui::theme;

/// Rows a jump to the first or the last row moves by: more than any list holds.
const FAR: isize = 1_000_000;

/// Rows the selection moves by for each key.
const MOVES: [(KeyCode, isize); 8] = [
    (KeyCode::ArrowUp, -1),
    (KeyCode::KeyW, -1),
    (KeyCode::ArrowDown, 1),
    (KeyCode::KeyS, 1),
    (KeyCode::PageUp, -(VISIBLE_ROWS as isize)),
    (KeyCode::PageDown, VISIBLE_ROWS as isize),
    (KeyCode::Home, -FAR),
    (KeyCode::End, FAR),
];

/// The action of each key. Enter plays through the focused row itself.
const ACTS: [(KeyCode, Act); 6] = [
    (KeyCode::KeyR, Act::Rename),
    (KeyCode::F2, Act::Rename),
    (KeyCode::KeyC, Act::Duplicate),
    (KeyCode::Delete, Act::Delete),
    (KeyCode::Backspace, Act::Delete),
    (KeyCode::KeyT, Act::Sort),
];

/// Selects the row the pointer gave focus to.
pub(super) fn follow_focus(focus: Res<InputFocus>, rows: Query<&Row>, mut worlds: ResMut<Worlds>) {
    let Some(row) = focus.get().and_then(|entity| rows.get(entity).ok()) else {
        return;
    };
    let index = worlds.window.index(worlds.list.len(), row.0);
    if let Some(index) = index.filter(|index| *index != worlds.window.cursor) {
        worlds.window.cursor = index;
    }
}

/// Opens the new world menu on N, one level deeper.
pub(super) fn new_world(
    keys: Res<ButtonInput<KeyCode>>,
    mut trail: ResMut<Trail>,
    mut menu: ResMut<NextState<Menu>>,
) {
    if keys.just_pressed(KeyCode::KeyN) {
        trail.0.push(Menu::Worlds);
        menu.set(Menu::NewGame);
    }
}

/// Moves the selection with the arrows, W and S, Page Up and Down, Home, End and the wheel, and
/// runs the action keys.
pub(super) fn browse(
    keys: Res<ButtonInput<KeyCode>>,
    wheel: Res<AccumulatedMouseScroll>,
    mut ctx: Ctx,
) {
    let pressed = MOVES.iter().filter(|(key, _)| keys.just_pressed(*key));
    let by: isize = pressed.map(|(_, by)| by).sum();
    if by != 0 {
        ctx.worlds.step(by);
    }
    let notches = match wheel.unit {
        MouseScrollUnit::Line => wheel.delta.y,
        MouseScrollUnit::Pixel => wheel.delta.y / theme::ROW_HEIGHT,
    };
    if notches.round() != 0.0 {
        let len = ctx.worlds.list.len();
        ctx.worlds
            .window
            .scroll(len, VISIBLE_ROWS, -notches.round() as isize);
    }
    if let Some((_, act)) = ACTS.iter().find(|(key, _)| keys.just_pressed(*key)) {
        ctx.run(*act);
    }
}

/// Gives keyboard focus to the row of the selected world whenever that row changes.
pub(super) fn focus_cursor(
    worlds: Res<Worlds>,
    rows: Query<(Entity, &Row)>,
    mut focus: ResMut<InputFocus>,
    mut last: Local<Option<(Entity, usize)>>,
) {
    if worlds.list.is_empty() {
        return;
    }
    let slot = worlds.window.cursor - worlds.window.top;
    let row = rows.iter().find(|(_, row)| row.0 == slot);
    let wanted = row.map(|(entity, _)| (entity, slot));
    if let Some((entity, _)) = wanted
        && (wanted != *last || focus.get().is_none())
    {
        *last = wanted;
        focus.set(entity, FocusCause::Navigated);
    }
}
