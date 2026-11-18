//! A tally of ground samples in test builds: how many times this thread shaped the ground of a sphere, so
//! that a test can state what a reader costs without timing it.

use std::cell::Cell;

use super::Globe;

thread_local! {
    /// Ground samples this thread has taken on any sphere.
    static TAKEN: Cell<u64> = const { Cell::new(0) };
}

/// Counts one ground sample of this thread.
pub fn count() {
    TAKEN.set(TAKEN.get() + 1);
}

impl Globe {
    /// Returns how many times this thread has shaped the ground of a sphere since it started.
    pub fn sampled() -> u64 {
        TAKEN.get()
    }
}
