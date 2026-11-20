//! The rule of the turn: the ship aligns before the drive spools, aims where the destination
//! will be when the ship gets there, and never turns while the drive runs.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::Ship;
use crate::ship::warp::{DROP_MARGIN, RAMP, SPOOL_TIME, Warp};

/// Largest angle in radians between the nose and the course at which the drive may spool.
const ANGLE: f64 = 1.0e-3;

/// Largest turn rate in radians per second at which the drive may spool.
const RATE: f64 = 1.0e-3;

/// Share of the lock radius a course in warp may miss by before the drive drops out to align again.
const OFF: f64 = 0.8;

/// Returns the seconds a ship in warp needs to cover `distance` metres up to a destination with a
/// lock sphere of `lock` metres, at most at `cap` metres per second, as the drive flies it: at
/// the cap while far, then slower in proportion to the clearance, which falls off as an
/// exponential until the drop-out margin.
pub fn travel(distance: f64, lock: f64, cap: f64) -> f64 {
    let clear = (distance - lock).max(0.0);
    let knee = cap * RAMP;
    let margin = (lock * DROP_MARGIN).max(1.0);
    let ramp = RAMP * (clear.min(knee) / margin).max(1.0).ln();
    (clear - knee).max(0.0) / cap + ramp
}

/// Returns the seconds from now to the arrival of a ship that flies `distance` metres to a
/// destination with a lock sphere of `lock` metres: the spool that is still to run plus the
/// cruise. The destination is aimed at where it will be then.
pub fn lead(warp: Warp, distance: f64, lock: f64, cap: f64) -> f64 {
    let spool = match warp {
        Warp::Off => SPOOL_TIME,
        Warp::Spooling { left } => left,
        Warp::Active { .. } => 0.0,
    };
    spool + travel(distance, lock, cap)
}

/// Returns true when the nose is within a milliradian of the course and the ship has nearly
/// stopped turning, so that the drive may spool once the residual is snapped away.
pub fn settled(ship: &Ship, course: DVec3) -> bool {
    ship.forward().angle_between(course) <= ANGLE && ship.spin.length() <= RATE
}

/// Turns the nose exactly onto the course and stops the spin, so that the heading held through
/// the spool and the warp is the course itself and not a residual the stick never closes.
pub fn snap(ship: &mut Ship, course: DVec3) {
    ship.rot = (DQuat::from_rotation_arc(ship.forward(), course) * ship.rot).normalize();
    ship.spin = DVec3::ZERO;
}

/// Returns true when a ship in warp flies a heading that would miss a destination `distance`
/// metres off along `course`, with a lock sphere of `lock` metres, by more than the drive can
/// make up: the drive drops out and the ship aligns again.
pub fn missed(ship: &Ship, course: DVec3, (distance, lock): (f64, f64)) -> bool {
    let angle = ship.forward().angle_between(course);
    matches!(ship.warp, Warp::Active { .. }) && angle.sin().abs() * distance > OFF * lock
}
