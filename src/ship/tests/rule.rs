//! Tests of the one speed rule of the ship: the top speed grows smoothly with the clearance from
//! the ground to open space, with no jump anywhere, up to its one cap.

use crate::ship::cruise::top_speed_at;
use crate::ship::tune::SPACE_CAP;

/// Metres between two clearances the rule is read at.
const STEP: f64 = 10.0;

/// From the ground to 1,000 km the top speed never falls and never grows by more than the
/// clearance it gains over 1.6 s, so it has no jump, also where it meets the space cap, and over
/// an Earth it reaches the cap at 48 km.
#[test]
fn the_speed_rule_is_continuous_in_the_clearance() {
    let orbital = 7_800.0;
    let mut last = top_speed_at(0.0, orbital);
    assert_eq!(last, 40.0);
    for step in 1..100_000 {
        let now = top_speed_at(f64::from(step) * STEP, orbital);
        assert!(
            now >= last && now - last <= STEP / 1.6 + 1e-9,
            "{last} to {now} at step {step}"
        );
        last = now;
    }
    assert_eq!(last, SPACE_CAP);
    assert!(top_speed_at(47_000.0, orbital) < SPACE_CAP);
    assert_eq!(top_speed_at(48_000.0, orbital), SPACE_CAP);
}
