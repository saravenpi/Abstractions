//! Tests of when the sea sprays and how much: per metre, by height and speed, and the ripples.

use super::tune::{DROPS, MAX_THROWN, SPRAY_PER_METRE};
use super::wash::{flow, ripple_speed, skim, thrown};

/// Returns the flecks thrown over `metres` of travel split into `frames` equal frames.
fn over(metres: f32, frames: u32) -> u32 {
    let mut due = 0.0;
    (0..frames)
        .map(|_| thrown(&mut due, SPRAY_PER_METRE * metres / frames as f32))
        .sum()
}

/// The same flight throws the same spray at 10 and at 120 frames a second, to a fleck.
#[test]
fn spray_per_distance_ignores_the_frame_rate() {
    let slow = over(10.0, 5);
    let fast = over(10.0, 60);
    assert!(slow.abs_diff(fast) <= 1, "{slow} against {fast}");
    assert!(slow.abs_diff((SPRAY_PER_METRE * 10.0) as u32) <= 1);
}

/// A wash tears spray off the sea only when it is low enough, and a fast skim only when it is
/// fast and low.
#[test]
fn spray_depends_on_height_and_speed() {
    let push = 90.0 * 22.0;
    let low = flow(push, 2.0, 1.2, (0.0, 1.0));
    let high = flow(push, 2.0, 1.2, (0.0, 40.0));
    assert!(low.spray > 0.0 && high.spray == 0.0);
    assert!(skim(12.0, 0.5, 1.0) > 0.0);
    assert_eq!(skim(1.0, 0.5, 1.0), 0.0);
    assert_eq!(skim(12.0, 30.0, 1.0), 0.0);
    assert_eq!(skim(12.0, -5.0, 1.0), 0.0);
}

/// Ripples run out faster under a heavier gravity, with the square root of it.
#[test]
fn ripple_speed_follows_gravity() {
    let ratio = ripple_speed(22.0) / ripple_speed(5.5);
    assert!((ratio - 2.0).abs() < 1e-4, "{ratio}");
    assert_eq!(ripple_speed(0.0), 0.0);
}

/// A frame never throws more than the cap, however much is owed, and the pool is larger.
#[test]
fn a_frame_throws_at_most_the_cap() {
    let mut due = 0.0;
    assert_eq!(thrown(&mut due, 1.0e6), MAX_THROWN);
    assert!(due < 1.0);
    assert!(MAX_THROWN as usize <= DROPS);
}
