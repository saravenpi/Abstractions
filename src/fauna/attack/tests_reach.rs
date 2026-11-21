//! Tests of the reach of a cut: a carcass at the robot's feet is in reach from a third person
//! camera metres behind it, and one past the robot's reach, or off the line of sight, is not.

use bevy::prelude::*;

use super::aim::sighted;
use super::tune::CUT_RANGE;

/// The robot of the tests, and a third person camera 3.5 m behind and above it looking down past it.
const ROBOT: Vec3 = Vec3::ZERO;

/// Returns the eyes of a camera behind the robot and the unit way they look at `at`.
fn view(at: Vec3) -> (Vec3, Vec3) {
    let eye = Vec3::new(0.4, 1.8, 3.5);
    (eye, (at - eye).normalize())
}

/// Looking at a carcass a metre in front of the robot from behind it, the part is in reach though
/// it lies farther than the reach from the camera; a part past the reach of the robot is not, and
/// neither is one far off the line of sight.
#[test]
fn a_carcass_at_the_feet_is_in_reach_in_third_person() {
    let part = Vec3::new(0.0, -0.6, -1.0);
    let sight = view(part);
    assert!(part.distance(sight.0) > CUT_RANGE);
    assert!(sighted(part, 0.3, sight, Some(ROBOT)).is_some());
    let far = Vec3::new(0.0, -0.6, -(CUT_RANGE + 2.0));
    assert!(sighted(far, 0.3, view(far), Some(ROBOT)).is_none());
    let aside = Vec3::new(1.5, -0.6, -1.0);
    assert!(sighted(aside, 0.3, sight, Some(ROBOT)).is_none());
}
