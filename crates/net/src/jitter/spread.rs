//! How irregularly datagrams arrive: the 95th percentile of recent arrival jitter.
//!
//! The jitter of a snapshot is how much the time between two arrivals differs from the time
//! between the two samples. A lost snapshot widens both gaps alike, and a body at rest that
//! sends slowly widens both too, so neither counts as jitter.

/// How many of the latest jitter measures the percentile is taken over: two seconds at the
/// full rate.
pub const WINDOW: usize = 64;

/// The latest jitter measures in a fixed ring, and the last snapshot kept.
#[derive(Debug, Clone)]
pub(super) struct Spread {
    /// Jitter measures in seconds.
    gaps: [f64; WINDOW],
    /// How many slots hold a measure.
    len: usize,
    /// The slot the next measure goes in.
    next: usize,
    /// Sample time and arrival time of the last snapshot kept.
    last: Option<(f64, f64)>,
}

impl Default for Spread {
    /// Returns a window with no measure and no snapshot kept.
    fn default() -> Self {
        Self {
            gaps: [0.0; WINDOW],
            len: 0,
            next: 0,
            last: None,
        }
    }
}

impl Spread {
    /// Measures the jitter of a snapshot sampled at `time` that arrived at `arrival`, against
    /// the last one kept. When `kept` is true the snapshot becomes the one to measure against.
    pub(super) fn record(&mut self, time: f64, arrival: f64, kept: bool) {
        if let Some((last_time, last_arrival)) = self.last {
            let gap = ((arrival - last_arrival) - (time - last_time)).abs();
            if gap.is_finite() {
                self.gaps[self.next] = gap;
                self.next = (self.next + 1) % WINDOW;
                self.len = (self.len + 1).min(WINDOW);
            }
        }
        if kept && arrival.is_finite() {
            self.last = Some((time, arrival));
        }
    }

    /// Returns the 95th percentile of the measures in seconds, or zero before the first one.
    pub(super) fn p95(&self) -> f64 {
        let mut gaps = self.gaps;
        let Some(rank) = (self.len * 95).div_ceil(100).checked_sub(1) else {
            return 0.0;
        };
        *gaps[..self.len]
            .select_nth_unstable_by(rank, f64::total_cmp)
            .1
    }

    /// Forgets the last snapshot kept, so the next one is not measured against another frame.
    pub(super) fn forget(&mut self) {
        self.last = None;
    }
}
