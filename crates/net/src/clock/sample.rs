//! One ping and its pong, and the offset it alone gives.
//!
//! A pong carries the universe time the server read when it answered. Assuming the trip out
//! and the trip back took the same time, the universe time at the moment the pong arrived is
//! that reading plus half the round trip.

/// One ping and its pong.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sample {
    /// Local monotonic time in seconds when the ping left.
    pub sent: f64,
    /// Local monotonic time in seconds when the pong arrived.
    pub received: f64,
    /// Universe time in seconds the server wrote in the pong.
    pub server: f64,
}

impl Sample {
    /// Returns the round trip in seconds.
    pub fn rtt(&self) -> f64 {
        self.received - self.sent
    }

    /// Returns what to add to a local time to get the universe time, by this sample alone.
    pub fn offset(&self) -> f64 {
        self.server + self.rtt() / 2.0 - self.received
    }
}
