//! Tests of the rule of the turn: tolerance, lead time and the miss that drops the drive.

use bevy::math::{DQuat, DVec3};

use super::align::{lead, missed, settled, tolerance, travel};
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
fn a_far_destination_needs_a_tighter_aim() {
    assert!(tolerance(1.0e12, 1.0e6) < tolerance(1.0e8, 1.0e6));
    assert!(tolerance(1.0e8, 1.0e6) <= 1.0e-3);
    assert!(tolerance(1.0, 1.0e6) <= 1.0e-3);
}

#[test]
fn the_drive_waits_for_the_nose_and_for_the_spin_to_settle() {
    let course = DVec3::NEG_Z;
    let far = (1.0e10, 1.0e7);
    assert!(settled(&facing(0.0), course, far));
    assert!(!settled(&facing(0.01), course, far));
    let mut turning = facing(0.0);
    turning.spin = DVec3::Y * 0.05;
    assert!(!settled(&turning, course, far));
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
    let course = DVec3::NEG_Z;
    let mut ship = facing(1.0e-4);
    let far = (1.0e11, 1.0e7);
    assert!(!missed(&ship, course, far));
    ship.warp = Warp::Active { speed: 1.0 };
    assert!(missed(&ship, course, far));
    assert!(!missed(&ship, course, (1.0e5, 1.0e7)));
}
