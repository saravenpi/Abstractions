//! The jitter buffer of one remote body: snapshots in, a smooth state a little in the past out.
//!
//! A snapshot is drawn at `now` minus the one-way delay and minus the jitter delay. The one-way
//! delay is the shortest arrival time minus sample time over the last [`super::WINDOW`]
//! arrivals, so snapshots are drawn as soon as they can arrive, whatever the latency.
//!
//! The delay is two send intervals plus the 95th percentile of arrival jitter, within
//! [`DELAY_MIN`] and [`DELAY_MAX`]. It moves toward that target by at most [`DRIFT`] per
//! second, so motion never stutters while it adapts. The send interval is the one the sender
//! uses while moving: a body at rest sends slower, but its samples still line up with its
//! arrivals, so the delay does not grow and the body holds still between them.

use super::lag::Lag;
use super::ring::Ring;
use super::spread::Spread;
use super::{Snap, View};

/// Shortest delay in seconds.
pub const DELAY_MIN: f64 = 0.05;

/// Longest delay in seconds.
pub const DELAY_MAX: f64 = 0.25;

/// Most the delay changes per second, in seconds.
pub const DRIFT: f64 = 0.005;

/// The snapshots of one remote body and the delay it is drawn at.
#[derive(Debug, Clone)]
pub struct Buffer<E> {
    /// The snapshots in time order.
    pub(super) ring: Ring<E>,
    /// The recent arrival jitter.
    pub(super) spread: Spread,
    /// The recent one-way delays.
    lag: Lag,
    /// Seconds between two snapshots while the body moves.
    interval: f64,
    /// The delay in use, in seconds.
    delay: f64,
    /// The `now` of the last [`Buffer::sample`].
    last_now: Option<f64>,
}

impl<E: Copy + Default> Buffer<E> {
    /// Returns an empty buffer for a body that sends a snapshot every `interval` seconds while
    /// it moves, such as 1/30. Its delay starts at the target for no jitter.
    pub fn new(interval: f64) -> Self {
        let mut buffer = Self {
            ring: Ring::default(),
            spread: Spread::default(),
            lag: Lag::default(),
            interval: interval.max(0.0),
            delay: 0.0,
            last_now: None,
        };
        buffer.delay = buffer.target();
        buffer
    }

    /// Adds a snapshot that arrived at universe time `arrival`, on the same clock as the `now`
    /// of [`Buffer::sample`]. Returns false when it was dropped as late or repeated; its
    /// arrival still counts toward the jitter and the one-way delay.
    pub fn push(&mut self, snap: Snap<E>, arrival: f64) -> bool {
        let kept = self.ring.push(snap);
        self.spread.record(snap.time, arrival, kept);
        self.lag.record(snap.time, arrival);
        kept
    }

    /// Moves the delay toward its target for the time since the last call, and returns the
    /// state at `now` minus the one-way delay and the delay. Returns nothing while the buffer
    /// is empty.
    pub fn sample(&mut self, now: f64) -> Option<View<E>> {
        let elapsed = self.last_now.map_or(0.0, |last| (now - last).max(0.0));
        self.last_now = Some(now);
        let step = DRIFT * elapsed;
        self.delay += (self.target() - self.delay).clamp(-step, step);
        self.ring.at(now - self.latency() - self.delay)
    }

    /// Returns the one-way delay in seconds: the shortest arrival time minus sample time over
    /// the recent arrivals, or zero before the first one.
    pub fn latency(&self) -> f64 {
        self.lag.shortest()
    }

    /// Returns the delay in use, in seconds.
    pub fn delay(&self) -> f64 {
        self.delay
    }

    /// Returns the delay the buffer moves toward: two send intervals plus the 95th percentile
    /// of arrival jitter, within [`DELAY_MIN`] and [`DELAY_MAX`].
    pub fn target(&self) -> f64 {
        (2.0 * self.interval + self.spread.p95()).clamp(DELAY_MIN, DELAY_MAX)
    }

    /// Returns how many snapshots are kept.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Returns true when no snapshot is kept.
    pub fn is_empty(&self) -> bool {
        self.ring.len() == 0
    }
}
