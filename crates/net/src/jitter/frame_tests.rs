//! Tests of a frame switch: converted snapshots draw the same body, dropped ones are gone.

use glam::DVec3;

use super::Buffer;
use super::buffer_tests::{INTERVAL, arrivals, circle};

/// Returns a buffer fed one second of a body on a circle.
fn fed() -> Buffer<()> {
    let mut buffer = Buffer::new(INTERVAL);
    for (arrival, snap) in arrivals(circle, INTERVAL, 0.02, 0.05, 1.0) {
        buffer.push(snap, arrival);
    }
    buffer
}

/// Moving every snapshot by a fixed offset draws the same state, moved by that offset, at the
/// same delay.
#[test]
fn a_rebase_moves_the_sample_by_the_offset() {
    let offset = DVec3::new(1.0e6, -250.5, 3.25);
    let (mut before, mut after) = (fed(), fed());
    after.rebase(|snap| {
        Some(super::Snap {
            position: snap.position + offset,
            ..snap
        })
    });
    assert_eq!((after.len(), after.delay()), (before.len(), before.delay()));
    assert_eq!(after.latency(), before.latency());
    for now in [0.9, 1.0, 1.05] {
        let (old, new) = (before.sample(now).unwrap(), after.sample(now).unwrap());
        assert!((new.position - (old.position + offset)).length() < 1e-6);
        assert_eq!((new.time, new.rotation), (old.time, old.rotation));
    }
}

/// Snapshots the conversion refuses are dropped, and the rest stay in order.
#[test]
fn a_rebase_drops_what_it_cannot_convert() {
    let mut buffer = fed();
    let kept = buffer.len();
    buffer.rebase(|snap| (snap.time >= 0.5).then_some(snap));
    assert!(buffer.len() < kept && !buffer.is_empty());
    assert!(buffer.sample(0.0).unwrap().time < 0.5);
    buffer.rebase(|_| None);
    assert!(buffer.is_empty() && buffer.sample(2.0).is_none());
}
