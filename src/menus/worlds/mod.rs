//! Worlds menu: the saved worlds in a table, with play, rename, duplicate, delete and sort.

mod act;
mod bars;
mod keys;
mod modal;
mod older;
mod ops;
mod rows;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_show;
mod view;
mod when;

use bevy::prelude::*;

use super::Menu;
use super::nav::MenuKeys;
use crate::saves::WorldEntry;
use crate::ui::Window;
use ops::SortBy;

/// Number of world rows drawn at once. A longer list scrolls through them.
const VISIBLE_ROWS: usize = 5;

/// One saved world with what the table shows besides the save itself.
#[derive(Debug, Clone, PartialEq)]
struct Listed {
    /// The world file.
    entry: WorldEntry,
    /// Where the player is: a planet and its system.
    place: String,
}

/// The saved worlds shown, their order, and which one is selected.
#[derive(Resource, Debug, Default)]
struct Worlds {
    /// Every world, in display order.
    list: Vec<Listed>,
    /// The selected row and the first drawn row.
    window: Window,
    /// How the list is ordered.
    sort: SortBy,
}

impl Worlds {
    /// Returns the selected world.
    fn selected(&self) -> Option<&Listed> {
        self.list.get(self.window.cursor)
    }

    /// Returns the world drawn in `slot`, counted from the top of the table.
    fn at(&self, slot: usize) -> Option<&Listed> {
        let index = self.window.index(self.list.len(), slot)?;
        self.list.get(index)
    }

    /// Moves the selection by `by` rows.
    fn step(&mut self, by: isize) {
        self.window.step(self.list.len(), VISIBLE_ROWS, by);
    }

    /// Replaces the list, orders it and selects the world with id `keep`, or keeps the cursor.
    fn fill(&mut self, mut list: Vec<Listed>, keep: Option<&str>) {
        ops::sort(&mut list, self.sort);
        let found = keep.and_then(|id| list.iter().position(|world| world.entry.id == id));
        self.list = list;
        self.window.cursor = found.unwrap_or(self.window.cursor);
        self.step(0);
    }

    /// Switches to the other order and keeps the selected world selected.
    fn resort(&mut self) {
        let keep = self.selected().map(|world| world.entry.id.clone());
        self.sort = self.sort.next();
        let list = std::mem::take(&mut self.list);
        self.fill(list, keep.as_deref());
    }
}

/// What the keyboard is doing in the menu.
#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub(super) enum Mode {
    /// Moving through the list.
    #[default]
    Browse,
    /// Typing a new name for the selected world. Holds the text typed so far.
    Rename(String),
    /// Asking whether to delete the selected world. Deleting needs `armed`, which starts off.
    Delete {
        /// Whether the delete choice is the one Enter applies.
        armed: bool,
    },
}

/// The short message under the table: why a name was refused, or what went wrong on disk.
#[derive(Resource, Debug, Default, PartialEq, Eq)]
struct Notice(String);

/// Run condition: a world name is being typed or a delete is being confirmed.
pub(super) fn editing(mode: Res<Mode>) -> bool {
    *mode != Mode::Browse
}

/// Registers the worlds menu systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Worlds>()
        .init_resource::<Mode>()
        .init_resource::<Notice>()
        .add_observer(act::on_activate)
        .add_systems(OnEnter(Menu::Worlds), view::spawn)
        .add_systems(
            Update,
            (
                (keys::follow_focus, keys::browse, keys::new_world)
                    .chain()
                    .in_set(MenuKeys),
                (modal::rename_keys, modal::delete_keys)
                    .after(MenuKeys)
                    .run_if(editing),
                keys::focus_cursor,
                older::report,
                rows::show,
                rows::show_lines,
                rows::show_bars,
            )
                .chain()
                .run_if(in_state(Menu::Worlds)),
        );
}
