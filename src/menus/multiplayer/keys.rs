//! Keyboard handling of the multiplayer menu while moving through the list. The arrows move the
//! focus between rows through the shared navigation, and the list follows the focus.

use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::*;

use super::Servers;
use super::act::{Act, Ctx};
use super::rows::Row;

/// The action of each key. Enter joins through the focused row itself.
const ACTS: [(KeyCode, Act); 7] = [
    (KeyCode::KeyD, Act::Direct),
    (KeyCode::KeyA, Act::Add),
    (KeyCode::KeyE, Act::Edit),
    (KeyCode::Delete, Act::Remove),
    (KeyCode::Backspace, Act::Remove),
    (KeyCode::F5, Act::Refresh),
    (KeyCode::KeyR, Act::Refresh),
];

/// Selects the row the keyboard or the pointer gave focus to.
pub(super) fn follow_focus(
    focus: Res<InputFocus>,
    rows: Query<&Row>,
    mut servers: ResMut<Servers>,
) {
    let Some(row) = focus.get().and_then(|entity| rows.get(entity).ok()) else {
        return;
    };
    if row.0 != servers.cursor && row.0 < servers.list.len() {
        servers.cursor = row.0;
    }
}

/// Runs the action keys.
pub(super) fn browse(keys: Res<ButtonInput<KeyCode>>, mut ctx: Ctx) {
    if let Some((_, act)) = ACTS.iter().find(|(key, _)| keys.just_pressed(*key)) {
        ctx.run(*act);
    }
}

/// Gives the focus to the row of the selected server when the selection moved or the focused
/// row is gone, as after the list was built again.
pub(super) fn focus_cursor(
    servers: Res<Servers>,
    rows: Query<(Entity, &Row)>,
    mut focus: ResMut<InputFocus>,
    mut last: Local<Option<usize>>,
) {
    let current = focus.get().and_then(|entity| rows.get(entity).ok());
    let current = current.map(|(_, row)| row.0);
    if current.is_some() && *last == Some(servers.cursor) {
        return;
    }
    let Some((entity, _)) = rows.iter().find(|(_, row)| row.0 == servers.cursor) else {
        return;
    };
    *last = Some(servers.cursor);
    if current != Some(servers.cursor) {
        focus.set(entity, FocusCause::Navigated);
    }
}
