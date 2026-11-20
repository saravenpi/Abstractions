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
