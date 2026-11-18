//! The three tables of the computer, from the nearest things to the farthest.

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
