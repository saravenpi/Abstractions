//! Worlds menu: the saved worlds sorted by last played, with play and delete.

mod keys;
mod rows;
mod view;

use bevy::prelude::*;

use super::Menu;
use super::nav::MenuKeys;
use crate::saves::WorldEntry;

/// Number of world rows visible at once.
const VISIBLE_ROWS: usize = 8;

/// First slot of the items below the rows. Rows use their own index.
const EXTRA_SLOT: usize = 10_000;

/// The saved worlds shown, most recently played first.
#[derive(Resource, Default)]
struct WorldList(Vec<WorldEntry>);

/// Index of the row waiting for a delete confirmation.
#[derive(Resource, Default)]
struct Pending(Option<usize>);

/// Index of the row that takes focus when the rows are built.
#[derive(Resource, Default)]
struct Selected(usize);

/// Index of the first visible row.
#[derive(Resource, Default, PartialEq)]
struct Scroll(usize);

/// A button that plays or deletes the world at this index.
#[derive(Component, Clone, Copy)]
struct Row(usize);

/// Registers the worlds menu systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<WorldList>()
        .init_resource::<Pending>()
        .init_resource::<Selected>()
        .init_resource::<Scroll>()
        .add_observer(keys::on_activate)
        .add_systems(OnEnter(Menu::Worlds), view::spawn)
        .add_systems(
            Update,
            (
                (keys::pending_keys, keys::escape).in_set(MenuKeys),
                rows::rebuild.run_if(resource_changed::<WorldList>),
                rows::scroll,
                rows::show_rows,
                view::show_hint,
            )
                .chain()
                .run_if(in_state(Menu::Worlds)),
        );
}
