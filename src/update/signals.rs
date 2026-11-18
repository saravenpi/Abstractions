//! What the game and a running install tell each other: bytes received, cancel and commit.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

/// The install runs and can still be cancelled.
const RUNNING: u8 = 0;
/// The player cancelled before the swap.
const CANCELLED: u8 = 1;
/// The install reached the swap: it can no longer be cancelled.
const COMMITTED: u8 = 2;

/// Shared between the game and one install. Cancel and commit exclude each other: whichever
/// comes first wins, so a cancel never leaves a half-made swap.
#[derive(Debug, Default)]
pub struct Signals {
    /// Bytes received so far.
    done: AtomicU64,
    /// One of [`RUNNING`], [`CANCELLED`], [`COMMITTED`].
    state: AtomicU8,
}

impl Signals {
    /// Records the number of bytes received so far.
    pub fn progress(&self, bytes: u64) {
        self.done.store(bytes, Ordering::Relaxed);
    }

    /// Returns the number of bytes received so far.
    pub fn done(&self) -> u64 {
        self.done.load(Ordering::Relaxed)
    }

    /// Moves from running to `to`, and returns whether it did.
    fn settle(&self, to: u8) -> bool {
        let moved = self
            .state
            .compare_exchange(RUNNING, to, Ordering::SeqCst, Ordering::SeqCst);
        moved.is_ok()
    }

    /// Cancels the install. Returns false when it is past the point of no return.
    pub fn cancel(&self) -> bool {
        self.settle(CANCELLED)
    }

    /// Returns whether the install was cancelled.
    pub fn cancelled(&self) -> bool {
        self.state.load(Ordering::SeqCst) == CANCELLED
    }

    /// Claims the swap. Returns false when the player cancelled first.
    pub fn commit(&self) -> bool {
        self.settle(COMMITTED)
    }
}
