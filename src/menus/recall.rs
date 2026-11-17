//! Remembering the entry last used in each menu, so that it takes the focus on the next visit.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use std::collections::HashMap;

use super::Menu;

/// Position of a menu entry among the entries of its menu.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry(pub usize);

/// The entry last used in each menu.
#[derive(Resource, Debug, Clone, Default)]
pub struct LastEntry(HashMap<Menu, usize>);

impl LastEntry {
    /// Remembers that entry `index` of `menu` was used.
    pub fn set(&mut self, menu: Menu, index: usize) {
        self.0.insert(menu, index);
    }

    /// Returns the entry last used in `menu`, or `fallback` when none was.
    pub fn of(&self, menu: Menu, fallback: usize) -> usize {
        self.0.get(&menu).copied().unwrap_or(fallback)
    }
}

/// Remembers an activated entry under the menu that is open.
pub(super) fn remember(
    event: On<Activate>,
    entries: Query<&Entry>,
    menu: Res<State<Menu>>,
    mut last: ResMut<LastEntry>,
) {
    if let Ok(entry) = entries.get(event.entity) {
        last.set(*menu.get(), entry.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each menu remembers its own entry and starts on the fallback.
    #[test]
    fn remembers_one_entry_per_menu() {
        let mut last = LastEntry::default();
        assert_eq!(last.of(Menu::Main, 1), 1);
        last.set(Menu::Main, 3);
        last.set(Menu::Pause, 2);
        last.set(Menu::Main, 4);
        assert_eq!((last.of(Menu::Main, 0), last.of(Menu::Pause, 0)), (4, 2));
    }
}
