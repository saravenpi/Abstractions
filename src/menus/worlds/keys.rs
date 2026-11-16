//! Keyboard and button handling of the worlds menu.

use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use super::{Pending, Row, Selected, WorldList};
use crate::menus::Menu;
use crate::saves::{SaveDir, Starter, delete};

/// State the row buttons read and write.
#[derive(SystemParam)]
pub(super) struct Ctx<'w, 's> {
    list: ResMut<'w, WorldList>,
    pending: ResMut<'w, Pending>,
    selected: ResMut<'w, Selected>,
    dir: Res<'w, SaveDir>,
    starter: Starter<'w, 's>,
}

impl Ctx<'_, '_> {
    /// Deletes the world file at `index` and drops it from the list.
    fn remove(&mut self, index: usize) {
        let Some(entry) = self.list.0.get(index) else {
            return;
        };
        match delete(&self.dir.0, &entry.id) {
            Ok(()) => {
                self.list.0.remove(index);
                self.selected.0 = index;
            }
            Err(error) => warn!("could not delete world {}: {error}", entry.id),
        }
    }
}

/// Plays a row, or deletes it when it is waiting for confirmation.
pub(super) fn on_activate(event: On<Activate>, rows: Query<&Row>, mut ctx: Ctx) {
    let Ok(Row(index)) = rows.get(event.entity) else {
        return;
    };
    if ctx.pending.0 == Some(*index) {
        ctx.pending.0 = None;
        ctx.remove(*index);
    } else if let Some(entry) = ctx.list.0.get(*index).cloned() {
        ctx.starter.start(entry);
    }
}

/// Asks for a delete confirmation on Delete or Backspace, and drops it when focus moves away.
pub(super) fn pending_keys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    rows: Query<&Row>,
    mut pending: ResMut<Pending>,
) {
    let current = focus
        .get()
        .and_then(|entity| rows.get(entity).ok())
        .map(|row| row.0);
    if pending.0.is_some() && pending.0 != current {
        pending.0 = None;
    }
    if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) && current.is_some() {
        pending.0 = current;
    }
}

/// Cancels a pending delete on Escape, or goes back to the main menu.
pub(super) fn escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<Pending>,
    mut menu: ResMut<NextState<Menu>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if pending.0.is_some() {
        pending.0 = None;
    } else {
        menu.set(Menu::Main);
    }
}
