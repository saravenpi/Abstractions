//! The warp drive: spools away from gravity wells, runs fast in the void and slows near bodies.

use bevy::math::DVec3;

use super::body::Ship;
use super::wells::{Well, nearest};
use crate::universe::GLY;

#[cfg(test)]
mod tests;

/// Seconds the drive spools before it engages.
pub const SPOOL_TIME: f64 = 4.0;

/// Seconds the drive needs to cross its clearance: speed is clearance over this.
const RAMP: f64 = 2.0;

/// Slowest warp speed in metres per second, so leaving the edge of a lock sphere is not endless.
const FLOOR: f64 = 2000.0;

/// Fastest warp speed between the stars of a galaxy, in metres per second.
pub const CRUISE_CAP: f64 = 5.0 * GLY;

/// Fastest warp speed of a long jump, across a galaxy or between galaxies, in metres per second.
pub const JUMP_CAP: f64 = 5000.0 * GLY;

/// Share of a lock radius outside the lock sphere at which a closing ship drops out.
const DROP_MARGIN: f64 = 0.1;

/// Radii from a body inside which it carries the warp bubble fully, and beyond which not at all.
const CARRY: (f64, f64) = (50.0, 100.0);

/// Returns the warp speed `speed`, in metres per second, as a share of the top speed of the
/// drive, from 0 to 1. Warp speeds span ten decades, so the share is taken on a logarithmic
/// scale, from the slowest warp to the fastest jump: a tenfold speed is always the same step.
pub fn fraction(speed: f64) -> f64 {
    ((speed / FLOOR).max(1.0).ln() / (JUMP_CAP / FLOOR).ln()).min(1.0)
}

/// State of the warp drive.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Warp {
    /// The drive is off.
    #[default]
    Off,
    /// The drive spools and engages when `left` seconds have passed.
    Spooling {
        /// Seconds until the drive engages.
        left: f64,
    },
    /// The ship moves along its heading at warp speed.
    Active {
        /// Current warp speed in metres per second.
        speed: f64,
    },
}

/// Returns the metres from `pos` to the nearest lock sphere, negative inside one, infinite without wells.
pub fn clearance(wells: &[Well], pos: DVec3) -> f64 {
    nearest(wells, pos).map_or(f64::INFINITY, |well| well.clearance(pos))
}

/// Returns the velocity of the nearest body, or zero without wells.
fn body_velocity(wells: &[Well], pos: DVec3) -> DVec3 {
    nearest(wells, pos).map_or(DVec3::ZERO, |well| well.vel)
}

/// Returns the velocity a nearby moving body gives the warp bubble, so a planet can be reached.
fn carry(wells: &[Well], pos: DVec3) -> DVec3 {
    let share = |well: &Well| {
        let radii = well.pos.distance(pos) / well.radius;
        ((CARRY.1 - radii) / (CARRY.1 - CARRY.0)).clamp(0.0, 1.0)
    };
    nearest(wells, pos).map_or(DVec3::ZERO, |well| well.vel * share(well))
}

/// Starts to spool the drive and returns true, or returns false inside a lock sphere or when not off.
pub fn request(ship: &mut Ship, wells: &[Well]) -> bool {
    let free = ship.warp == Warp::Off && !ship.landed && clearance(wells, ship.pos) > 0.0;
    if free {
        ship.warp = Warp::Spooling { left: SPOOL_TIME };
    }
    free
}

/// Stops the drive. A ship that was in warp is left at rest relative to the body that is
/// nearest now, so it arrives beside its destination and stays there: the velocity it had
/// when it engaged belonged to the place it left, and would carry it away from this one. A
/// ship that was not in warp keeps its own velocity.
pub fn drop_out(ship: &mut Ship, wells: &[Well]) {
    if matches!(ship.warp, Warp::Active { .. }) {
        ship.vel = body_velocity(wells, ship.pos);
    }
    ship.warp = Warp::Off;
}

/// Moves a ship in warp along its heading for `dt` seconds, at most at `cap` metres per second.
///
/// Speed is the clearance over the ramp time, so it falls to zero at the edge of a lock sphere, and
/// one move never covers more than half of the clearance. Returns the speed, or nothing when the
/// ship closes on a lock sphere and must drop out.
fn cruise(ship: &mut Ship, wells: &[Well], cap: f64, dt: f64) -> Option<f64> {
    let before = clearance(wells, ship.pos);
    if before <= 0.0 {
        return None;
    }
    let speed = (before / RAMP).clamp(FLOOR, cap).min(before * 0.5 / dt);
    ship.pos += (ship.forward() * speed + carry(wells, ship.pos)) * dt;
    let after = clearance(wells, ship.pos);
    let margin = nearest(wells, ship.pos).map_or(0.0, |well| well.lock * DROP_MARGIN);
    (after > 0.0 && (after >= before || after > margin)).then_some(speed)
}

/// Runs the drive for `dt` seconds: counts the spool down, engages, moves the ship and drops out.
///
/// A spool is cancelled inside a lock sphere. The ship keeps no velocity of its own while in warp.
pub fn advance(ship: &mut Ship, wells: &[Well], cap: f64, dt: f64) {
    match ship.warp {
        Warp::Off => {}
        Warp::Spooling { .. } if clearance(wells, ship.pos) <= 0.0 => ship.warp = Warp::Off,
        Warp::Spooling { left } if left > dt => ship.warp = Warp::Spooling { left: left - dt },
        Warp::Spooling { .. } => ship.warp = Warp::Active { speed: 0.0 },
        Warp::Active { .. } => match cruise(ship, wells, cap, dt) {
            Some(speed) => ship.warp = Warp::Active { speed },
            None => drop_out(ship, wells),
        },
    }
}
