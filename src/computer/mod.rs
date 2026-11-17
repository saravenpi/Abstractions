//! The board computer: the atlas of discovered systems and galaxies, and fast travel.

mod atlas;
mod board;
mod format;
mod keys;
mod locate;
mod open;
mod reveal;
mod route;
mod rows;
mod scene;
mod show;
mod track;
mod view;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_tables;

pub use atlas::Atlas;
pub use board::{Board, Tab, VISIBLE_ROWS};
pub use format::{distance, gravity};
pub use keys::ENGAGE_KEY;
pub use locate::{SYSTEM_REACH, ship_position, target_position, whereabouts};
pub use open::{Computer, OPEN_KEY};
pub use reveal::{HOME_NEIGHBOURS, REVEAL_CELLS, REVEAL_GLY};
pub use route::{Destination, Engage, RouteStatus, Target};
pub use rows::{Line, View};
pub use track::Known;

use bevy::prelude::*;

/// The systems that read the keyboard, the wheel and the pointer of the open screen.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct Input;

/// Registers the computer resources and systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Destination>()
        .init_resource::<RouteStatus>()
        .init_resource::<Board>()
        .init_resource::<Known>()
        .add_plugins((open::plugin, track::plugin, keys::plugin, show::plugin));
}
