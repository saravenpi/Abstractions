//! How long a trip takes: the time the warp drive needs to cover a distance, worked out in
//! closed form from the law of the drive and the three speed tiers of the ship.
//!
//! The drive flies at the clearance over [`RAMP`] seconds, from the edge of the lock sphere it
//! leaves to the one it reaches, and never faster than the cap of its tier. The clearance
//! grows and falls by a factor of e every `RAMP` seconds, so a ramp from a clearance `c` to the
//! speed `v` takes `RAMP * ln(v * RAMP / c)` seconds and covers `v * RAMP` metres. The ship
//! flies the rest at the cap. No law of nature sets the tiers: they come from the trip times the
//! owner wants, 1 to 2 minutes to the nearest star and 15 minutes to the next galaxy.

use super::route::Target;
use crate::ship::{CRUISE_CAP, GALAXY_CAP, JUMP_CAP, JUMP_DISTANCE, RAMP, SPOOL_TIME};
use crate::universe::{ORBIT, SUN_RADIUS};

/// Clearance in metres from which a trip starts: the widest orbit of a planet, 40 AU, where a
/// ship leaving a system has left its planets behind.
pub const LEAVE_CLEARANCE: f64 = ORBIT.1;

/// Clearance in metres at which the drive drops out at a star: a tenth of the lock radius of
/// the Sun, which is five solar radii.
pub const ARRIVE_CLEARANCE: f64 = 0.5 * SUN_RADIUS;

/// Returns the cap in metres per second of the tier a trip of `gap` metres flies at: the
/// galaxy cap between galaxies, the cruise cap up to [`JUMP_DISTANCE`] and the jump cap beyond.
fn cap(gap: f64, same_galaxy: bool) -> f64 {
    match (same_galaxy, gap > JUMP_DISTANCE) {
        (false, _) => GALAXY_CAP,
        (true, true) => JUMP_CAP,
        (true, false) => CRUISE_CAP,
    }
}

/// Returns the seconds the drive needs for a trip of `gap` metres: the spool, the ramp out of
/// the system left, the flight at the peak speed and the ramp down into the place reached. A
/// trip too short to reach its cap peaks at the speed whose two ramps cover it.
pub fn trip_seconds(gap: f64, same_galaxy: bool) -> f64 {
    let gap = gap.max(0.0);
    let peak = cap(gap, same_galaxy).min(gap / (2.0 * RAMP));
    let ramp = |from: f64| RAMP * (peak * RAMP / from).ln().max(0.0);
    let cruise = (gap - 2.0 * peak * RAMP).max(0.0) / peak.max(1.0);
    SPOOL_TIME + ramp(LEAVE_CLEARANCE) + ramp(ARRIVE_CLEARANCE) + cruise
}

/// Returns the average speed in metres per second of a trip of `gap` metres, so that the
/// gap over it is [`trip_seconds`]. The card gives it to [`super::arrival`] for a time.
pub fn pace(gap: f64, same_galaxy: bool) -> f64 {
    gap.max(0.0) / trip_seconds(gap, same_galaxy)
}

/// Returns true when `target` lies in the galaxy the ship is inside, given as the third of the
/// places `here` of the ship. A ship outside every galaxy flies the galaxy tier.
pub fn same_galaxy(here: [Option<Target>; 3], target: Target) -> bool {
    let galaxy = match target {
        Target::Body(id) => id.star.galaxy,
        Target::Star(id) => id.galaxy,
        Target::Galaxy(id) => id,
    };
    here[2] == Some(Target::Galaxy(galaxy))
}
