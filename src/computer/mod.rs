//! The board computer: the atlas of discovered systems and galaxies, fast travel, and the map
//! screen that shows them from a planet out to the universe.

mod acting;
mod arrow;
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
mod home;
mod hover;
mod keys;
mod labels;
mod layout;
mod legend;
mod lens;
mod locate;
mod map;
mod moons;
mod open;
mod orbits;
mod pointer;
mod readout;
mod results;
mod reveal;
mod rim;
mod route;
mod rows;
mod scene;
mod season;
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
mod tests_climate;
#[cfg(test)]
mod tests_close;
#[cfg(test)]
mod tests_detail;
#[cfg(test)]
mod tests_giant;
#[cfg(test)]
mod tests_home;
#[cfg(test)]
mod tests_homeward;
#[cfg(test)]
mod tests_labels;
#[cfg(test)]
mod tests_lens;
#[cfg(test)]
mod tests_moons;
#[cfg(test)]
mod tests_nav;
#[cfg(test)]
mod tests_panel;
#[cfg(test)]
mod tests_pointer;
#[cfg(test)]
mod tests_rig;
#[cfg(test)]
mod tests_rim;
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
mod tests_width;
#[cfg(test)]
mod tests_words;

pub use arrow::{arrow, bearing};
pub use atlas::Atlas;
pub use board::{Board, VISIBLE_ROWS};
pub use chart::{Dot, FILL, GRID, chart, hit, mark, span};
pub use course::{Act, action};
pub use draw::side;
pub use format::{distance, gravity};
pub use home::{arranged, away, beacon, homes, homeward, pinned, tab_of, toward};
pub use labels::{keep, priority, room, scale_bar};
pub use layout::GLYPH;
pub use lens::{Lens, MAX_ZOOM};
pub use locate::{SYSTEM_REACH, here, ship_position, target_position, whereabouts};
pub use map::{ChartLayer, Field, SIZE, Sheet, body, field, frame, name, ring, ship, spot};
pub use moons::{MOON_STEP, MOON_ZOOM, is_moon, seen};
pub use open::Computer;
pub use orbits::satellites;
pub use pointer::{DOUBLE_CLICK, MapView, REACH, RESET_KEY, SLOP, ZOOM_STEP};
pub use reveal::{HOME_NEIGHBOURS, REVEAL_CELLS, REVEAL_GLY};
pub use rim::{APART, home_row, rank, rim, spread};
pub use route::{Block, Destination, Engage, HOME_LABEL, RouteStatus, Target};
pub use rows::{Line, View};
pub use season::{climate_line, direction};
pub use semantic::{DIVE, entry, shift};
pub use sift::Sort;
pub use tab::Tab;
pub use track::Known;
pub use typing::Typing;
pub use view::{Area, SearchBox};
pub use words::temperature;

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
