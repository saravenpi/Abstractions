//! Send rates: 30 records a second while moving at any frame rate, 2 at rest, and the first
//! record of a thing that starts moving at once.

use crate::net::state::rate::Cadence;

/// Returns how many records one thing sends in `seconds` at `fps` frames a second, at rest or not.
fn sent(fps: u32, seconds: u32, at_rest: bool) -> usize {
    let mut cadence = Cadence::default();
    let frames = fps * seconds;
    (0..frames)
        .filter(|frame| cadence.due(f64::from(*frame) / f64::from(fps), at_rest))
        .count()
}

/// A moving thing sends 30 records a second, at 60, 144 and 50 frames a second.
#[test]
fn moving_sends_thirty_a_second() {
    for fps in [60, 144, 50, 240] {
        let count = sent(fps, 10, false);
        assert!((299..=301).contains(&count), "{count} records at {fps} fps");
    }
}

/// A thing at rest sends 2 records a second.
#[test]
fn resting_sends_two_a_second() {
    for fps in [60, 144] {
        let count = sent(fps, 10, true);
        assert!((19..=21).contains(&count), "{count} records at {fps} fps");
    }
}

/// A thing at rest that starts moving is sent at once, not half a second later.
#[test]
fn waking_sends_at_once() {
    let mut cadence = Cadence::default();
    assert!(cadence.due(0.0, true));
    assert!(!cadence.due(0.1, true));
    assert!(cadence.due(0.11, false));
    assert!(!cadence.due(0.12, false));
    assert!(cadence.due(0.11 + 1.0 / 30.0, false));
}
