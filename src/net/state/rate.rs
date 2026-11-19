//! How often a thing is sent: 30 times a second while it moves, twice a second at rest.

/// Seconds between two records of a moving thing.
pub const MOVING: f64 = 1.0 / 30.0;

/// Seconds between two records of a thing at rest.
pub const RESTING: f64 = 0.5;

/// Speed in metres per second under which a thing counts as at rest.
pub const REST_SPEED: f64 = 0.05;

/// When the next record of one thing is due.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cadence {
    /// Local time in seconds the next record is due at, once one was sent.
    next: Option<f64>,
    /// Whether the last record said the thing was at rest.
    resting: bool,
}

impl Cadence {
    /// Returns whether a record is due at local time `now` for a thing that is `at_rest`, and
    /// counts it as sent when it is. A thing that starts moving is sent at once. The next
    /// time moves by whole intervals, so a frame rate that does not divide the rate keeps the
    /// average, and it never falls more than one interval behind.
    pub fn due(&mut self, now: f64, at_rest: bool) -> bool {
        let interval = if at_rest { RESTING } else { MOVING };
        let woke = self.resting && !at_rest;
        let next = match self.next {
            Some(next) if !woke && now < next => return false,
            Some(next) if !woke && now - next < interval => next,
            _ => now,
        };
        self.next = Some(next + interval);
        self.resting = at_rest;
        true
    }
}
