//! The state of the computer screen: the open tab, its rows, the cursor and the scroll offset.

use bevy::prelude::*;

use super::rows::Line;

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
    /// Index of the selected row.
    pub cursor: usize,
    /// Index of the first drawn row.
    pub top: usize,
    /// True when the rows must be built again.
    pub stale: bool,
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
        let last = self.lines.len().saturating_sub(1);
        let deepest = self.lines.len().saturating_sub(VISIBLE_ROWS);
        self.cursor = self.cursor.saturating_add_signed(by).min(last);
        let lowest = (self.cursor + 1).saturating_sub(VISIBLE_ROWS);
        self.top = self.top.min(deepest).min(self.cursor).max(lowest);
    }

    /// Scrolls by `by` rows, stopping at both ends, and drags the cursor to keep it drawn.
    pub fn scroll(&mut self, by: isize) {
        let last = self.lines.len().saturating_sub(1);
        let deepest = self.lines.len().saturating_sub(VISIBLE_ROWS);
        self.top = self.top.saturating_add_signed(by).min(deepest);
        let bottom = self.top + VISIBLE_ROWS - 1;
        self.cursor = self.cursor.clamp(self.top, bottom).min(last);
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
