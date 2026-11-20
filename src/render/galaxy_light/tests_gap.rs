//! Test of the wait between two bakes of the galaxy light.

use super::GalaxyLight;
use super::tune::MIN_GAP;

/// The first bake never waits, and the next waits the minimum gap after the last was taken.
#[test]
fn bakes_wait_a_minimum_between_them() {
    let mut light = GalaxyLight::default();
    assert!(light.rested(0.0));
    light.finished = Some(10.0);
    assert!(!light.rested(10.0 + 0.5 * MIN_GAP));
    assert!(light.rested(10.0 + MIN_GAP));
}
