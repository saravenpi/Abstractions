//! The rule of the turn: the ship aligns before the drive spools, aims where the destination
//! will be when the ship gets there, and never turns while the drive runs.

use bevy::math::{DQuat, DVec3};

use crate::ship::body::Ship;
use crate::ship::warp::{DROP_MARGIN, FLOOR, RAMP, SPOOL_TIME, Warp};

/// Largest angle in radians between the nose and the course at which the drive may spool.
const ANGLE: f64 = 1.0e-3;

/// Largest turn rate in radians per second at which the drive may spool.
const RATE: f64 = 1.0e-3;

/// Lock radii from the destination beyond which a ship that has flown past it drops out. Nearer
/// it the route ends by itself, as the ship comes within its arrival distance.
const PAST: f64 = 2.0;

/// Times the lead is taken again from the place it ends at: the second pass moves it by less
/// than a millisecond of the time of flight.
const PASSES: usize = 2;

/// Lock radii from the destination within which the heading is no longer corrected: the last stretch
/// takes seconds, in which a destination that moves has gone a few hundredths of its lock radius, and
/// a new aim each time the target point moved a little is what spools the drive again and again.
const CLOSE: f64 = 20.0;

/// Share of the lock radius a course in warp may miss by before the drive drops out to align again.
const OFF: f64 = 0.8;

/// Returns the seconds a ship in warp needs to cover `distance` metres up to a destination with a
/// lock sphere of `lock` metres, at most at `cap` metres per second, as the drive flies it: at
/// the cap while far, then slower in proportion to the clearance, which falls off as an
/// exponential, and at the slowest speed of the drive for what is left of it until the drop-out
/// margin. A small lock sphere is mostly that last stretch.
pub fn travel(distance: f64, lock: f64, cap: f64) -> f64 {
    let clear = (distance - lock).max(0.0);
    let knee = cap * RAMP;
    let margin = (lock * DROP_MARGIN).max(1.0);
    let top = clear.min(knee);
    let slow = (FLOOR * RAMP).max(margin);
    let ramp = RAMP * (top / slow).max(1.0).ln();
    let crawl = (top.min(slow) - margin).max(0.0) / FLOOR;
    (clear - knee).max(0.0) / cap + ramp + crawl
}

/// Returns the seconds from now to the arrival of a ship that flies `distance` metres to a
/// destination with a lock sphere of `lock` metres: the spool that is still to run plus the
/// cruise. The destination is aimed at where it will be then.
pub fn lead(warp: Warp, distance: f64, lock: f64, cap: f64) -> f64 {
    spool(warp) + travel(distance, lock, cap)
}

/// Returns the vector from the ship to where a destination that moves will be when the ship
/// gets there, with a lock sphere of `lock` metres, flying at most at `cap` metres per second.
/// `at` gives the vector to the destination `ahead` seconds from now. The time of the flight
/// depends on the distance to the place it ends at, so the lead is taken again from that place.
pub fn led(
    warp: Warp,
    (lock, cap): (f64, f64),
    at: impl Fn(f64) -> Option<DVec3>,
) -> Option<DVec3> {
    let mut to = at(0.0)?;
    for _ in 0..PASSES {
        to = at(lead(warp, to.length(), lock, cap))?;
    }
    Some(to)
}

/// Returns the seconds the drive still has to spool before it carries the ship: the ship keeps
/// its own velocity until then, and has none once the drive runs.
pub fn spool(warp: Warp) -> f64 {
    match warp {
        Warp::Off => SPOOL_TIME,
        Warp::Spooling { left } => left,
        Warp::Active { .. } => 0.0,
    }
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

/// Returns true when a ship in warp has flown its leg and the drive drops out: it has flown past
/// `way`, the point beside a body in the way that it was sent to, or without one it flies a
/// heading that would miss a destination `to` metres off, with a lock sphere of `lock` metres, by
/// more than the drive can make up. The ship then aligns again. The miss is the length of the cross
/// product of the heading and the course: the angle between them, taken from an arc cosine, cannot
/// be told from zero under 1.5e-8 radians, which is thousands of light-years across a galaxy.
pub fn missed(ship: &Ship, way: Option<DVec3>, (to, lock): (DVec3, f64)) -> bool {
    let off = match way {
        Some(at) => (at - ship.pos).dot(ship.forward()) <= 0.0,
        None => {
            let behind = ship.forward().dot(to) < 0.0;
            let aside = ship.forward().cross(to).length();
            let closing = to.length() <= CLOSE * lock;
            (behind && to.length() > PAST * lock) || (aside > OFF * lock && !closing)
        }
    };
    matches!(ship.warp, Warp::Active { .. }) && off
}
