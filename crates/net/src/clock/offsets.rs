//! The offset between the local clock and the server's universe clock, from the latest pings.
//!
//! The sample with the shortest round trip waited least in queues, so its estimate is the one
//! used.

use super::Sample;

/// How many of the latest samples are kept.
pub const SAMPLES: usize = 16;

/// Longest round trip in seconds a sample may have. A longer one is thrown away as absurd.
pub const RTT_MAX: f64 = 5.0;

/// Weight of a new round trip in the smoothed round trip, as in TCP.
const SMOOTHING: f64 = 0.125;

/// The last [`SAMPLES`] samples in a fixed ring, and the round trips the HUD shows. It never
/// allocates.
#[derive(Debug, Clone, Default)]
pub struct Offsets {
    /// The samples, the oldest overwritten first once the ring is full.
    ring: [Sample; SAMPLES],
    /// How many slots of the ring hold a sample.
    len: usize,
    /// The slot the next sample goes in.
    next: usize,
    /// Moving average of every kept round trip, in seconds.
    smoothed: Option<f64>,
}

impl Offsets {
    /// Returns an empty ring.
    pub fn new() -> Self {
        Self::default()
    }

    /// Keeps `sample` and returns true, unless its round trip is negative, over [`RTT_MAX`] or
    /// not a number, or its offset is not finite: then it is thrown away and false is returned.
    pub fn push(&mut self, sample: Sample) -> bool {
        let rtt = sample.rtt();
        if !(0.0..=RTT_MAX).contains(&rtt) || !sample.offset().is_finite() {
            return false;
        }
        self.ring[self.next] = sample;
        self.next = (self.next + 1) % SAMPLES;
        self.len = (self.len + 1).min(SAMPLES);
        self.smoothed = Some(
            self.smoothed
                .map_or(rtt, |old| old + (rtt - old) * SMOOTHING),
        );
        true
    }

    /// Returns how many samples are kept, at most [`SAMPLES`].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true before the first sample is kept.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the kept sample with the shortest round trip.
    pub fn best(&self) -> Option<Sample> {
        self.ring[..self.len]
            .iter()
            .copied()
            .min_by(|a, b| a.rtt().total_cmp(&b.rtt()))
    }

    /// Returns what to add to a local time to get the universe time: the offset of
    /// [`Offsets::best`].
    pub fn offset(&self) -> Option<f64> {
        self.best().map(|sample| sample.offset())
    }

    /// Returns the shortest kept round trip in seconds.
    pub fn rtt_min(&self) -> Option<f64> {
        self.best().map(|sample| sample.rtt())
    }

    /// Returns the moving average of the round trip in seconds, for the HUD and the server list.
    pub fn rtt_smoothed(&self) -> Option<f64> {
        self.smoothed
    }
}
