//! The warp speed tiers: the only tuned numbers of the drive.
//!
//! Faster-than-light travel has no physics to take a speed from, so each cap is set from the time
//! the owner wants a trip of its kind to take. The law that runs below the cap is the same in every
//! tier (see `warp::law`), so the trip times follow from the caps alone. They are the times of a
//! flight that leaves the edge of a system 40 AU from its star and drops out at the arrival distance
//! of the destination star, with the 4 s spool: the test `warp::tests_trip` integrates the law and
//! holds each tier to its band.
//!
//! | Tier | Used when | Cap | In c | Trip |
//! | --- | --- | --- | --- | --- |
//! | cruise | same galaxy, up to 50 ly | 0.15 ly/s | 4.7e6 c | nearest star (3.5 ly) in 60 s, 10 ly in 104 s |
//! | jump | same galaxy, over 50 ly | 190 ly/s | 6.0e9 c | 100,000 ly in 592 s |
//! | galaxy | another galaxy | 3,600 ly/s | 1.1e11 c | 3 million ly in 911 s |

use crate::universe::LY;

/// Fastest warp speed between the stars of a galaxy on a route of up to [`JUMP_DISTANCE`], in
/// metres per second: 0.15 light-years per second, 4.7 million times the speed of light.
pub const CRUISE_CAP: f64 = 0.15 * LY;

/// Fastest warp speed of a long jump inside a galaxy, in metres per second: 190 light-years per
/// second, 6.0 billion times the speed of light.
pub const JUMP_CAP: f64 = 190.0 * LY;

/// Fastest warp speed of a jump to another galaxy, in metres per second: 3,600 light-years per
/// second, 110 billion times the speed of light. It is the top of the speed scale of the drive.
pub const GALAXY_CAP: f64 = 3600.0 * LY;

/// Distance in metres beyond which a route inside a galaxy is a long jump: faster, and blind to
/// the stars on the way. It is 50 light-years.
pub const JUMP_DISTANCE: f64 = 50.0 * LY;
