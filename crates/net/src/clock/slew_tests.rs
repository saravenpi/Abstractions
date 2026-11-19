//! Tests of the slewed clock: it snaps on join and past a second, and otherwise slews forward.

use super::{Clock, SLEW, Sample};

/// Frame length of the tests, in seconds.
const FRAME: f64 = 1.0 / 60.0;

/// Returns true when `a` and `b` differ by less than a nanosecond.
fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Returns a clock that heard one pong putting the universe `offset` seconds ahead.
fn heard(offset: f64) -> Clock {
    let mut clock = Clock::new();
    assert!(clock.pong(Sample {
        sent: 10.0,
        received: 10.1,
        server: 10.05 + offset,
    }));
    clock
}

/// With no estimate the time shown passes through, and the first estimate snaps even when close.
#[test]
fn the_first_estimate_snaps() {
    let mut clock = Clock::new();
    assert_eq!(clock.advance(42.0, FRAME, 11.0), 42.0);
    let mut clock = heard(1000.0);
    assert!(near(clock.advance(1010.8, FRAME, 11.0), 1011.0));
}

/// An error of 1.5 s snaps, ahead or behind.
#[test]
fn an_error_over_a_second_snaps() {
    let mut clock = heard(1000.0);
    clock.advance(0.0, FRAME, 11.0);
    assert!(near(clock.advance(1012.0 - 1.5, FRAME, 12.0), 1012.0));
    assert!(near(clock.advance(1013.0 + 1.5, FRAME, 13.0), 1013.0));
}

/// Runs a clock `error` seconds off the estimate for `frames` frames and returns the error left.
/// Each frame must move forward by the frame length give or take [`SLEW`] of it.
fn slew(error: f64, frames: u32) -> f64 {
    let mut clock = heard(1000.0);
    let mut now = 11.0;
    clock.advance(0.0, FRAME, now);
    let mut shown = now + 1000.0 - error;
    for _ in 0..frames {
        now += FRAME;
        let next = clock.advance(shown + FRAME, FRAME, now);
        let step = next - shown;
        assert!(step > 0.0, "time went back by {step}");
        assert!((step - FRAME).abs() <= SLEW * FRAME + 1e-9, "step {step}");
        shown = next;
    }
    now + 1000.0 - shown
}

/// A 0.5 s error closes by 2 percent of each frame and time never runs backwards.
#[test]
fn a_half_second_error_closes_at_two_percent() {
    let frames = 600;
    let closed = SLEW * FRAME * f64::from(frames);
    assert!((slew(0.5, frames) - (0.5 - closed)).abs() < 1e-6);
    assert!((slew(-0.5, frames) - (-0.5 + closed)).abs() < 1e-6);
    assert!(slew(0.5, 1600).abs() < 1e-6);
}
