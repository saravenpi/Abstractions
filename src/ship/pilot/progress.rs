//! How a route comes on, counted in spools of the drive: a route that spools again and again without coming
//! nearer is dropped, so that it never loops.

use super::Autopilot;

/// Share of the nearest distance a spool must take off the distance to count as progress.
const GAIN: f64 = 0.01;

/// Spools of the drive a route may start without coming nearer before it is dropped.
pub(in crate::ship) const SPOOLS: u32 = 8;

/// How a route comes on, counted in spools of the drive.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(in crate::ship) struct Progress {
    /// The least distance in metres to the destination at the start of a spool.
    pub(super) best: Option<f64>,
    /// Spools started since the distance last fell by [`GAIN`].
    pub(super) spools: u32,
}

impl Autopilot {
    /// Counts a spool of the drive that starts `gap` metres from the destination, and returns true
    /// when [`SPOOLS`] of them in a row came no nearer: the route goes nowhere.
    pub(in crate::ship) fn stalled(&mut self, gap: f64) -> bool {
        let progress = &mut self.progress;
        if progress.best.is_none_or(|best| gap < best * (1.0 - GAIN)) {
            *progress = Progress {
                best: Some(gap),
                spools: 0,
            };
            return false;
        }
        progress.spools += 1;
        progress.spools > SPOOLS
    }
}
