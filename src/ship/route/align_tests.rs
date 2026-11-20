//! Tests of the rule of the turn: tolerance, lead time and the miss that drops the drive.

use bevy::math::{DQuat, DVec3};

use super::align::{lead, missed, settled, snap, travel};
use crate::ship::Ship;
use crate::ship::warp::{CRUISE_CAP, SPOOL_TIME, Warp};
use crate::space::Frame;
use crate::universe::StarId;

/// Returns a ship at the origin facing down the course `-Z` turned by `angle` radians about Y.
fn facing(angle: f64) -> Ship {
    let rot = DQuat::from_rotation_y(angle);
    Ship::at_rest(Frame::System(StarId::default()), DVec3::ZERO, rot)
}

#[test]
fn the_drive_waits_for_the_nose_and_for_the_spin_to_settle() {
    let course = DVec3::NEG_Z;
    assert!(settled(&facing(5.0e-4), course));
    assert!(!settled(&facing(0.01), course));
    let mut turning = facing(0.0);
    turning.spin = DVec3::Y * 0.05;
    assert!(!settled(&turning, course));
}

#[test]
fn a_far_flight_takes_the_distance_over_the_cap() {
    let distance = CRUISE_CAP * 1000.0;
    let time = travel(distance, 1.0e7, CRUISE_CAP);
    assert!(time > 990.0 && time < 1100.0, "{time}");
    assert!(lead(Warp::Off, distance, 1.0e7, CRUISE_CAP) - time - SPOOL_TIME < 1.0e-9);
}

#[test]
fn a_miss_only_counts_in_warp() {
    let mut ship = facing(1.0e-4);
    let far = (DVec3::NEG_Z * 1.0e11, 1.0e7);
    assert!(!missed(&ship, None, far));
    ship.warp = Warp::Active { speed: 1.0 };
    assert!(missed(&ship, None, far));
    assert!(!missed(&ship, None, (DVec3::NEG_Z * 1.0e5, 1.0e7)));
}

#[test]
fn snapping_puts_the_nose_on_the_course_and_stops_the_spin() {
    let mut ship = facing(5.0e-4);
    ship.spin = DVec3::Y * 1.0e-4;
    snap(&mut ship, DVec3::NEG_Z);
    assert!(ship.forward().angle_between(DVec3::NEG_Z) < 1.0e-12);
    assert_eq!(ship.spin, DVec3::ZERO);
}

#[test]
fn a_leg_ends_once_its_point_is_behind_the_ship() {
    let mut ship = facing(0.0);
    ship.warp = Warp::Active { speed: 1.0 };
    let near = (DVec3::NEG_Z, 1.0e7);
    assert!(!missed(&ship, Some(DVec3::NEG_Z * 10.0), near));
    assert!(missed(&ship, Some(DVec3::Z * 10.0), near));
    assert!(missed(&ship, None, (DVec3::Z * 1.0e8, 1.0e7)));
    assert!(!missed(&ship, None, (DVec3::Z * 1.0e7, 1.0e7)));
}
