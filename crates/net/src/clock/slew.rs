//! The universe clock a player shows: it follows the server without jumps or reversals.
//!
//! Each frame the game says which universe time it would show and how long the frame took.
//! The clock answers with the time to show instead. It snaps to the estimate on the first
//! one (the join) and when it is off by more than [`SNAP`]. Otherwise it runs at most
//! [`SLEW`] faster or slower than real time until it meets the estimate, so time never runs
//! backwards.

use super::{Offsets, Sample};

/// Error in seconds beyond which the clock snaps instead of slewing.
pub const SNAP: f64 = 1.0;

/// Largest share of a frame's length the clock adds or removes to close its error.
pub const SLEW: f64 = 0.02;

/// The universe clock of a player: the offset estimate and whether it has snapped once.
#[derive(Debug, Clone, Default)]
pub struct Clock {
    /// The ping samples and their estimate.
    offsets: Offsets,
    /// True once the clock has snapped to a first estimate.
    synced: bool,
}

impl Clock {
    /// Returns a clock with no estimate yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds the clock a pong. Returns false when the sample was thrown away.
    pub fn pong(&mut self, sample: Sample) -> bool {
        self.offsets.push(sample)
    }

    /// Returns the samples, for the round trips the HUD and the server list show.
    pub fn offsets(&self) -> &Offsets {
        &self.offsets
    }

    /// Returns the universe time at local monotonic time `now`, by the estimate alone.
    pub fn estimate(&self, now: f64) -> Option<f64> {
        self.offsets.offset().map(|offset| now + offset)
    }

    /// Returns the universe time to show this frame. `shown` is the time the game would show,
    /// the last one plus `delta`, and `now` is the local monotonic time. With no estimate yet
    /// it returns `shown`. It returns the estimate on the first call that has one and when the
    /// error is over [`SNAP`]. Otherwise it moves `shown` toward the estimate by at most
    /// [`SLEW`] times `delta`, so the time shown never goes back.
    pub fn advance(&mut self, shown: f64, delta: f64, now: f64) -> f64 {
        let Some(target) = self.estimate(now) else {
            return shown;
        };
        let error = target - shown;
        if !self.synced || error.abs() > SNAP {
            self.synced = true;
            return target;
        }
        let step = SLEW * delta.max(0.0);
        shown + error.clamp(-step, step)
    }
}
