//! Stretching the budget: more streamed work per frame while a queue waits and the frames are
//! short, within what the main thread may give.

/// Tasks waiting from which the budget of a frame of play starts to stretch, and from which it
/// is stretched the most.
const QUEUE: (f32, f32) = (6.0, 30.0);

/// Most times the budget of a frame of play stretches while work waits.
const MOST: f32 = 3.0;

/// Seconds of a frame above which the game has no time to spare and nothing stretches.
const TIGHT: f64 = 1.0 / 45.0;

/// How far the budget of this frame stretches: more gets done per frame while work waits and
/// the frames are short, and never more than the main thread may give.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stretch(f32);

impl Default for Stretch {
    /// Returns the plain budget.
    fn default() -> Self {
        Self(1.0)
    }
}

impl Stretch {
    /// Returns the stretch of a frame with `waiting` tasks out, after a frame of `frame` seconds.
    pub fn of(waiting: usize, frame: f64) -> Self {
        let long = ((waiting as f32 - QUEUE.0) / (QUEUE.1 - QUEUE.0)).clamp(0.0, 1.0);
        Self(1.0 + (MOST - 1.0) * long * f32::from(u8::from(frame < TIGHT)))
    }

    /// Returns the stretched milliseconds of a share of `millis`.
    pub fn millis(self, millis: f32) -> f32 {
        millis * self.0
    }

    /// Returns the stretched count of a budget of `items` per frame.
    pub fn count(self, items: usize) -> usize {
        (items as f32 * self.0).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The budget stretches with the queue, up to its limit, and not at all in a long frame.
    #[test]
    fn budget_stretches_with_the_queue() {
        let frame = 1.0 / 60.0;
        assert_eq!(Stretch::of(0, frame), Stretch::default());
        assert_eq!(Stretch::of(6, frame).millis(1.5), 1.5);
        assert_eq!(Stretch::of(18, frame).count(2), 4);
        assert_eq!(Stretch::of(30, frame).millis(1.5), 4.5);
        assert_eq!(Stretch::of(500, frame).count(8), 24);
        assert_eq!(Stretch::of(500, 1.0 / 30.0), Stretch::default());
    }
}
