//! Tests of stopping a route: the target goes, and the drive that runs drops out on the next frame.

use crate::computer::Target;
use crate::ship::pilot::{Autopilot, Phase};
use crate::universe::Universe;

/// Stopping a route that is flown drops the target and asks for the drop-out of the drive, which
/// the route flies out on its next frame. Stopping an idle autopilot asks for nothing.
#[test]
fn stopping_drops_the_target_and_asks_for_the_drop_out() {
    let universe = Universe::new(7);
    let mut pilot = Autopilot::default();
    pilot.engage(Target::Body(universe.home()));
    assert!(pilot.target().is_some());
    pilot.stop();
    assert_eq!(pilot.target(), None);
    assert_eq!(pilot.phase(), Phase::Idle);
    assert!(pilot.halt);
    let mut idle = Autopilot::default();
    idle.stop();
    assert!(!idle.halt);
}

/// A new route to the same target goes on as it is, and one to another target starts again.
#[test]
fn engaging_the_same_target_goes_on() {
    let universe = Universe::new(7);
    let home = Target::Body(universe.home());
    let star = Target::Star(universe.home().star);
    let mut pilot = Autopilot::default();
    pilot.engage(home);
    let before = pilot.clone();
    pilot.engage(home);
    assert_eq!(pilot, before);
    pilot.engage(star);
    assert_eq!(pilot.target(), Some(star));
}
