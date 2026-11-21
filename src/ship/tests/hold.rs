//! Tests of the hold that keeps a pilot in the seat until the ground beside the hatch is solid.

use crate::ship::hold::{HOLD, hold};

/// Solid ground lets the pilot out at once, and loose ground holds the pilot for [`HOLD`] seconds and no
/// longer.
#[test]
fn the_pilot_is_held_until_the_ground_is_solid_or_the_time_is_up() {
    assert_eq!(hold(0.0, true, 0.016), None);
    let mut held = Some(0.0);
    let mut frames = 0;
    while let Some(seconds) = held {
        held = hold(seconds, false, 0.5);
        frames += 1;
    }
    assert_eq!(frames, (HOLD / 0.5) as usize + 1);
    assert_eq!(hold(3.0, true, 0.5), None);
    assert_eq!(hold(3.0, false, 0.5), Some(3.5));
}
