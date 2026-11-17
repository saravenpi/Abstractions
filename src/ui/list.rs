//! The window of a long list: which row is selected and which rows are drawn.
//!
//! A list draws a fixed pool of rows, however many items it holds. The window says which items
//! the pool shows.

/// The selected row and the first drawn row of a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Window {
    /// Index of the selected row.
    pub cursor: usize,
    /// Index of the first drawn row.
    pub top: usize,
}

impl Window {
    /// Moves the cursor by `by` rows inside a list of `len` rows, stopping at both ends, and
    /// scrolls to keep it among the `visible` drawn rows.
    pub fn step(&mut self, len: usize, visible: usize, by: isize) {
        let last = len.saturating_sub(1);
        let deepest = len.saturating_sub(visible);
        self.cursor = self.cursor.saturating_add_signed(by).min(last);
        let lowest = (self.cursor + 1).saturating_sub(visible);
        self.top = self.top.min(deepest).min(self.cursor).max(lowest);
    }

    /// Scrolls by `by` rows, stopping at both ends, and drags the cursor to keep it drawn.
    pub fn scroll(&mut self, len: usize, visible: usize, by: isize) {
        let last = len.saturating_sub(1);
        let deepest = len.saturating_sub(visible);
        self.top = self.top.saturating_add_signed(by).min(deepest);
        let bottom = self.top + visible.max(1) - 1;
        self.cursor = self.cursor.clamp(self.top, bottom).min(last);
    }

    /// Returns the index of the row drawn in `slot`, counted from the top, when the list of
    /// `len` rows has one there.
    pub fn index(&self, len: usize, slot: usize) -> Option<usize> {
        Some(self.top + slot).filter(|index| *index < len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cursor stays inside the list and inside the drawn rows, whatever the moves.
    #[test]
    fn the_cursor_stays_drawn() {
        let mut window = Window::default();
        for by in [1, 30, -7, 1_000_000, -3, -1_000_000, 99, 5, -5] {
            window.step(100, 6, by);
            assert!((window.top..window.top + 6).contains(&window.cursor));
            assert!(window.top + 6 <= 100);
        }
        window.scroll(100, 6, -50);
        assert_eq!((window.cursor, window.top), (49, 44));
        window.step(3, 6, 0);
        assert_eq!((window.cursor, window.top), (2, 0));
        assert_eq!((window.index(3, 2), window.index(3, 3)), (Some(2), None));
        window.step(0, 6, 5);
        assert_eq!((window.cursor, window.top), (0, 0));
    }
}
