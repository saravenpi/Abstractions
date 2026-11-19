//! Tests of the one-way delay: a long trip is measured, so the body is drawn as smoothly as
//! on a short one.

use super::Buffer;
use super::buffer_tests::{INTERVAL, LATENCY, arrivals, circle, play};

/// Shortest one-way trip of this network in seconds: 150 ms less 20 ms of jitter.
const FAR: f64 = 0.13;

/// At 150 ms of latency with 20 ms of jitter either way and 5 percent loss, a body on a circle
/// at 10 m/s is drawn within 2 cm of its path and moves on every frame.
#[test]
fn a_far_body_is_drawn_smoothly() {
    let mut buffer = Buffer::new(INTERVAL);
    let list: Vec<_> = arrivals(circle, INTERVAL, 0.04, 0.05, 30.0)
        .into_iter()
        .map(|(arrival, snap)| (arrival - LATENCY + FAR, snap))
        .collect();
    let (mut worst, mut frozen, mut last) = (0.0_f64, 0, None);
    play(&mut buffer, &list, 30.0, |view, now, _| {
        if now > 1.0 {
            worst = worst.max(view.position.distance(circle(view.time).position));
            let moved = last.map_or(1.0, |at: glam::DVec3| at.distance(view.position));
            frozen += usize::from(moved < 0.01);
        }
        last = Some(view.position);
    });
    assert!(
        (buffer.latency() - FAR).abs() < 0.005,
        "{}",
        buffer.latency()
    );
    assert!(worst < 0.02, "worst error {worst} m");
    assert_eq!(frozen, 0);
}
