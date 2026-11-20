//! The trip times of the three warp tiers, from the speed law integrated step by step, and the
//! rules that keep a fast ship out of a body and its speed without a step.

use super::law::speed;
use super::{CRUISE_CAP, FLOOR, GALAXY_CAP, JUMP_CAP, RAMP, SPOOL_TIME};
use crate::ship::wells::STAR_LOCK;
use crate::universe::{AU, GRAVITATION, LY, SUN_MASS, SUN_RADIUS};

/// Seconds of one simulated step.
const DT: f64 = 1.0 / 30.0;

/// Pull in metres per second squared at the distance where the autopilot drops out at a star.
const SAFE_PULL: f64 = 0.5;

/// What one simulated warp did.
struct Trip {
    /// Seconds from the request of the drive to the drop-out.
    time: f64,
    /// Largest share by which the speed rose in one step.
    rise: f64,
    /// Least clearance in metres from a lock sphere over the flight.
    least: f64,
}

/// Flies a ship `distance` metres at most at `cap` metres per second: it leaves a star of the Sun
/// 40 AU out, with the spool before, and drops out at the arrival distance of a star of the Sun.
fn fly(distance: f64, cap: f64) -> Trip {
    let lock = SUN_RADIUS * STAR_LOCK;
    let arrival = (GRAVITATION * SUN_MASS / SAFE_PULL).sqrt();
    let mut trip = Trip {
        time: SPOOL_TIME,
        rise: 1.0,
        least: f64::INFINITY,
    };
    let (mut flown, mut last) = (0.0, 0.0);
    while distance - flown > arrival {
        let clearance = (flown + 40.0 * AU).min(distance - flown - lock);
        trip.least = trip.least.min(clearance);
        let now = speed(clearance, last, cap, DT);
        if last > 0.0 {
            trip.rise = trip.rise.max(now / last);
        }
        (flown, last, trip.time) = (flown + now * DT, now, trip.time + DT);
    }
    trip
}

/// The nearest star, 3.5 light-years away, takes 55 to 100 seconds and 10 light-years under two
/// minutes, at the cruise tier.
#[test]
fn the_nearest_star_is_about_a_minute_away() {
    let near = fly(3.5 * LY, CRUISE_CAP).time;
    assert!((55.0..100.0).contains(&near), "{near} s");
    let ten = fly(10.0 * LY, CRUISE_CAP).time;
    assert!((60.0..120.0).contains(&ten), "{ten} s");
    assert!(near < ten);
}

/// A jump across a galaxy, 100,000 light-years, takes 540 to 660 seconds.
#[test]
fn a_galaxy_is_ten_minutes_across() {
    let time = fly(1.0e5 * LY, JUMP_CAP).time;
    assert!((540.0..660.0).contains(&time), "{time} s");
}

/// A jump to another galaxy, 3 million light-years, takes 840 to 960 seconds.
#[test]
fn the_next_galaxy_is_fifteen_minutes_away() {
    let time = fly(3.0e6 * LY, GALAXY_CAP).time;
    assert!((840.0..960.0).contains(&time), "{time} s");
}

/// The speed never rises faster than the law that slows it, in any tier, and no flight comes
/// within a lock sphere: the ship always arrives outside it.
#[test]
fn the_speed_never_steps_and_the_ship_never_enters_a_lock() {
    let step = (DT / RAMP).exp() * (1.0 + 1e-9);
    for (distance, cap) in [
        (3.5 * LY, CRUISE_CAP),
        (1.0e5 * LY, JUMP_CAP),
        (3.0e6 * LY, GALAXY_CAP),
    ] {
        let trip = fly(distance, cap);
        assert!(trip.rise <= step, "rose by {}", trip.rise);
        assert!(trip.least > 0.0, "inside a lock sphere");
    }
}

/// A faster tier on a ship already in warp never lifts its speed by a step: the ship that
/// leaves a system for the open galaxy, or flies into a farther tier, climbs by the same rate.
#[test]
fn a_new_tier_is_reached_by_the_same_rise() {
    let last = 1.0e13;
    let now = speed(f64::INFINITY, last, JUMP_CAP, DT);
    assert!((now / last - (DT / RAMP).exp()).abs() < 1e-12);
    assert_eq!(speed(f64::INFINITY, 0.0, JUMP_CAP, DT), JUMP_CAP);
    assert_eq!(speed(f64::INFINITY, 1.0, CRUISE_CAP, DT), FLOOR);
}

/// The drop-out rule at every scale: one step never covers more than half of the clearance, and
/// the distance to stop, the speed times the ramp, is never more than the clearance once the
/// clearance is wider than the slowest speed of the drive covers in a ramp. So no speed overshoots
/// into a body.
#[test]
fn no_speed_overshoots_the_clearance() {
    for cap in [CRUISE_CAP, JUMP_CAP, GALAXY_CAP] {
        for power in 0..24 {
            let clearance = 10f64.powi(power);
            let now = speed(clearance, cap, cap, DT);
            assert!(now * DT <= clearance * 0.5 * (1.0 + 1e-12), "{clearance}");
            assert!(
                power < 4 || now * RAMP <= clearance * (1.0 + 1e-12),
                "{clearance}"
            );
        }
    }
}
