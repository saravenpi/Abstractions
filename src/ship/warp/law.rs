//! The speed law of the drive, the same in every tier: only the cap differs.

use super::{FLOOR, RAMP};

/// Returns the warp speed in metres per second of a ship `clearance` metres from the nearest lock
/// sphere, that flew at `last` metres per second a step before (0 when the drive has just
/// engaged), at most at `cap`, over a step of `dt` seconds.
///
/// The speed is the clearance over [`RAMP`], so it falls to nothing at the edge of a lock sphere
/// whatever the speed: the distance the ship needs to stop is never more than the clearance, so no
/// speed overshoots into a body. One step never covers more than half of the clearance, however
/// long the frame. The speed rises at most by the factor `e` per `RAMP` seconds, the rate at
/// which the clearance law itself falls, so it never steps up when the nearest body changes, when
/// the ship leaves a system for the open galaxy or when the tier of the route is faster. It never
/// falls below [`FLOOR`] or rises above `cap`.
pub fn speed(clearance: f64, last: f64, cap: f64, dt: f64) -> f64 {
    let closing = clearance * (1.0 - (-dt / RAMP).exp()) / dt;
    let rise = match last > 0.0 {
        true => (last * (dt / RAMP).exp()).max(FLOOR),
        false => f64::INFINITY,
    };
    closing
        .clamp(FLOOR, cap)
        .min(rise)
        .min(clearance * 0.5 / dt)
}
