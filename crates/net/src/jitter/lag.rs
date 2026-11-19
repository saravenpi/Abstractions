//! How long a snapshot takes to come: the shortest of the recent arrival times minus sample
//! times.
//!
//! Both times are universe times, so the shortest difference is the one-way trip of a datagram
//! that met no queue, plus whatever error the two clocks still share. Late arrivals only add to
//! the others, so the minimum ignores them.

use super::WINDOW;

/// The latest arrival delays in a fixed ring.
#[derive(Debug, Clone)]
pub(super) struct Lag {
    /// Arrival time minus sample time, in seconds.
    trips: [f64; WINDOW],
    /// How many slots hold a measure.
    len: usize,
    /// The slot the next measure goes in.
    next: usize,
}

impl Default for Lag {
    /// Returns a ring with no measure.
    fn default() -> Self {
        Self {
            trips: [0.0; WINDOW],
            len: 0,
            next: 0,
        }
    }
}

impl Lag {
    /// Measures a snapshot sampled at `time` that arrived at `arrival`. A measure that is not
    /// finite is ignored.
    pub(super) fn record(&mut self, time: f64, arrival: f64) {
        let trip = arrival - time;
        if !trip.is_finite() {
            return;
        }
        self.trips[self.next] = trip;
        self.next = (self.next + 1) % WINDOW;
        self.len = (self.len + 1).min(WINDOW);
    }

    /// Returns the shortest measure in seconds, or zero before the first one. It can be below
    /// zero when the receiver's clock runs behind the sender's.
    pub(super) fn shortest(&self) -> f64 {
        self.trips[..self.len]
            .iter()
            .copied()
            .reduce(f64::min)
            .unwrap_or(0.0)
    }
}
