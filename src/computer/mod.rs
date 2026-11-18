//! The board computer: the atlas of discovered systems and galaxies, and fast travel.

mod atlas;
mod board;
mod chart;
mod chrome;
mod course;
mod detail;
mod draw;
mod format;
mod giant;
mod glide;
mod keys;
mod labels;
mod layout;
mod lens;
mod locate;
mod map;
mod moons;
mod open;
mod orbits;
mod pointer;
mod readout;
mod reveal;
mod route;
mod rows;
mod scene;
mod semantic;
mod show;
mod sift;
mod tab;
mod track;
mod typing;
mod view;
mod words;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_detail;
#[cfg(test)]
mod tests_giant;
#[cfg(test)]
mod tests_labels;
#[cfg(test)]
mod tests_lens;
#[cfg(test)]
mod tests_moons;
#[cfg(test)]
mod tests_panel;
#[cfg(test)]
mod tests_pointer;
#[cfg(test)]
mod tests_rig;
#[cfg(test)]
mod tests_screen;
#[cfg(test)]
mod tests_semantic;
#[cfg(test)]
mod tests_sift;
#[cfg(test)]
mod tests_tables;
#[cfg(test)]
mod tests_wheel;
#[cfg(test)]
mod tests_words;

pub use atlas::Atlas;
pub use board::{Board, VISIBLE_ROWS};
pub use chart::{Dot, GRID, chart, hit, mark, span};
pub use draw::side;
pub use format::{distance, gravity};
pub use labels::{keep, priority, room, scale_bar};
pub use lens::{Lens, MAX_ZOOM};
pub use locate::{SYSTEM_REACH, ship_position, target_position, whereabouts};
pub use map::{Field, SIZE, Sheet, body, field, frame, name, ring, ship, spot};
pub use moons::{MOON_STEP, MOON_ZOOM, is_moon, seen};
pub use open::Computer;
pub use orbits::satellites;
pub use reveal::{HOME_NEIGHBOURS, REVEAL_CELLS, REVEAL_GLY};
pub use route::{Destination, Engage, RouteStatus, Target};
pub use rows::{Line, View};
pub use semantic::{DIVE, entry, shift};
pub use sift::Sort;
pub use tab::Tab;
pub use track::Known;
pub use view::Act;

use bevy::prelude::*;

/// The systems that read the keyboard, the wheel and the pointer of the open screen.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct Input;

/// Registers what the computer knows and how it learns it, without its screen: the
/// destination, the route status, the chart and the discovery systems.
pub(super) fn logic(app: &mut App) {
    app.init_resource::<Destination>()
        .init_resource::<RouteStatus>()
        .init_resource::<Board>()
        .init_resource::<Known>()
        .add_plugins(track::plugin);
}

/// Registers the computer resources and systems.
pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        logic,
        open::plugin,
        keys::plugin,
        pointer::plugin,
        show::plugin,
    ));
}
