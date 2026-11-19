//! The state of the computer screen: the open tab, its rows, the cursor and the scroll offset.

use bevy::prelude::*;

use super::route::Target;
use super::rows::Line;
use super::sift::Sort;
use super::tab::Tab;
use crate::space::UniPos;
use crate::ui::Window;
use crate::universe::PlanetSpec;

/// Most rows drawn at once. A longer table scrolls through them.
pub const VISIBLE_ROWS: usize = 12;

/// What the computer screen shows.
#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Board {
    /// The open tab.
    pub tab: Tab,
    /// Title of the table, such as the name of the current system.
    pub heading: String,
    /// Every row of the table. Only [`VISIBLE_ROWS`] of them are drawn.
    pub lines: Vec<Line>,
    /// For each row, the planet whose record is known, for the readout and the map.
    pub worlds: Vec<Option<PlanetSpec>>,
    /// Index of the selected row.
    pub cursor: usize,
    /// Index of the first drawn row.
    pub top: usize,
    /// True when the rows must be built again.
    pub stale: bool,
    /// Where the ship was when the rows were built, for the map.
    pub ship: UniPos,
    /// Rows the window has room for, or 0 before the panel was sized.
    pub fit: usize,
    /// The letters typed to narrow the list to the names that hold them.
    pub filter: String,
    /// How the rows are ordered.
    pub sort: Sort,
    /// The planet, the system and the galaxy the ship is inside.
    pub here: [Option<Target>; 3],
    /// The home planet, its star and its galaxy, in the same order.
    pub home: [Option<Target>; 3],
    /// The next place on the way home and where its centre is, or nothing on the home planet.
    pub beacon: Option<(Target, UniPos)>,
    /// A row the cursor moves to the next time the rows are replaced, when the table has it.
    pub seek: Option<Target>,
    /// The star or the galaxy whose chart the tab shows, or nothing for the one of the ship.
    pub inside: Option<Target>,
}

impl Board {
    /// Returns how many rows are drawn at once: what the window has room for, and never
    /// more than [`VISIBLE_ROWS`].
    pub fn window(&self) -> usize {
        match self.fit {
            0 => VISIBLE_ROWS,
            fit => fit.min(VISIBLE_ROWS),
        }
    }

    /// Opens `tab` at its first row and asks for its rows.
    pub fn show(&mut self, tab: Tab) {
        *self = Board {
            tab,
            stale: true,
            fit: self.fit,
            sort: self.sort,
            ..default()
        };
    }

    /// Opens `tab` on the chart of `inside`, a star for the system tab or a galaxy for the
    /// galaxy tab, or on the place of the ship for nothing.
    pub fn open(&mut self, tab: Tab, inside: Option<Target>) {
        self.show(tab);
        self.inside = inside;
    }

    /// Replaces the rows, keeping the cursor and the scroll offset inside the new table, then
    /// moves the cursor to the row that was asked for, if any.
    pub fn fill(&mut self, heading: String, lines: Vec<Line>) {
        self.heading = heading;
        self.lines = lines;
        self.stale = false;
        self.step(0);
        let wanted = |line: &Line| Some(line.target) == self.seek;
        if let Some(row) = self.lines.iter().position(wanted) {
            self.step(row as isize - self.cursor as isize);
        }
        self.seek = None;
    }

    /// Moves the cursor by `by` rows, stopping at both ends, and scrolls to keep it drawn.
    pub fn step(&mut self, by: isize) {
        let mut window = Window {
            cursor: self.cursor,
            top: self.top,
        };
        window.step(self.lines.len(), self.window(), by);
        (self.cursor, self.top) = (window.cursor, window.top);
    }

    /// Scrolls by `by` rows, stopping at both ends, and drags the cursor to keep it drawn.
    pub fn scroll(&mut self, by: isize) {
        let mut window = Window {
            cursor: self.cursor,
            top: self.top,
        };
        window.scroll(self.lines.len(), self.window(), by);
        (self.cursor, self.top) = (window.cursor, window.top);
    }

    /// Returns the row drawn in `slot`, counted from the top of the screen.
    pub fn at(&self, slot: usize) -> Option<&Line> {
        self.lines.get(self.top + slot)
    }

    /// Moves the cursor to the row drawn in `slot`, when there is one.
    pub fn pick(&mut self, slot: usize) {
        if self.at(slot).is_some() {
            self.cursor = self.top + slot;
        }
    }
}
