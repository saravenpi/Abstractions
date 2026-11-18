//! The state of the computer screen: the open tab, its rows, the cursor and the scroll offset.

use bevy::prelude::*;

use super::rows::Line;
use crate::space::UniPos;
use crate::ui::Window;
use crate::universe::PlanetSpec;

/// Number of rows drawn at once. A longer table scrolls through them.
pub const VISIBLE_ROWS: usize = 12;

/// The three tables of the computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// The star and planets of the current system.
    #[default]
    System,
    /// The known systems of the current galaxy.
    Galaxy,
    /// The known galaxies.
    Universe,
}

impl Tab {
    /// Every tab, in display order.
    pub const ALL: [Tab; 3] = [Tab::System, Tab::Galaxy, Tab::Universe];

    /// Returns the name shown on the tab.
    pub fn label(self) -> &'static str {
        match self {
            Tab::System => "System",
            Tab::Galaxy => "Galaxy",
            Tab::Universe => "Universe",
        }
    }

    /// Returns the tab `by` places to the right, wrapping around.
    pub fn step(self, by: isize) -> Tab {
        let count = Tab::ALL.len() as isize;
        Tab::ALL[(self as isize + by).rem_euclid(count) as usize]
    }
}

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
}

impl Board {
    /// Opens `tab` at its first row and asks for its rows.
    pub fn show(&mut self, tab: Tab) {
        *self = Board {
            tab,
            stale: true,
            ..default()
        };
    }

    /// Replaces the rows, keeping the cursor and the scroll offset inside the new table.
    pub fn fill(&mut self, heading: String, lines: Vec<Line>) {
        self.heading = heading;
        self.lines = lines;
        self.stale = false;
        self.step(0);
    }

    /// Moves the cursor by `by` rows, stopping at both ends, and scrolls to keep it drawn.
    pub fn step(&mut self, by: isize) {
        let mut window = Window {
            cursor: self.cursor,
            top: self.top,
        };
        window.step(self.lines.len(), VISIBLE_ROWS, by);
        (self.cursor, self.top) = (window.cursor, window.top);
    }

    /// Scrolls by `by` rows, stopping at both ends, and drags the cursor to keep it drawn.
    pub fn scroll(&mut self, by: isize) {
        let mut window = Window {
            cursor: self.cursor,
            top: self.top,
        };
        window.scroll(self.lines.len(), VISIBLE_ROWS, by);
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
