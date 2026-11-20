//! Tests of the hearing: calls fade with distance and carry farther for big animals, and no frame
//! sends more than [`CALLS`] calls.

use super::hear::level;
use super::stage::app;
use super::tune::CALLS;
use crate::fauna::stage::run;

/// A call is quieter farther away and louder from a bigger animal at the same distance.
#[test]
fn calls_fade_with_distance_and_carry_with_size() {
    assert!(level(40.0, 10.0) > level(40.0, 80.0));
    assert!(level(400.0, 80.0) > level(40.0, 80.0));
    assert!((level(1.0, 12.0) - 0.5).abs() < 1e-6);
}

/// Over two minutes in a herd, the hearing sends some calls and never more than [`CALLS`] a frame.
#[test]
fn calls_per_frame_are_bounded() {
    let (mut app, _) = app(true);
    run(&mut app, 120.0, 0.25);
    let sent = &app.world().resource::<super::stage::Sent>().0;
    let total: usize = sent.iter().sum();
    println!("{total} calls in {} frames", sent.len());
    assert!(sent.iter().all(|n| *n <= CALLS));
    assert!(total > 0, "a herd stayed silent for two minutes");
}
