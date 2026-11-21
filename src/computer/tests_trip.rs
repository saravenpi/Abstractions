//! Computer tests: the time a trip takes from the warp tiers of the ship, and which tier a
//! trip flies at.

use bevy::math::IVec3;

use super::trip::{pace, same_galaxy, trip_seconds};
use super::*;
use crate::ship::{JUMP_DISTANCE, SPOOL_TIME};
use crate::universe::{BodyId, GALAXY_CELL, GalaxyId, LY, StarId};

/// A trip to a star a few light-years away takes about a minute and a trip over ten
/// light-years a little more, since the cruise cap is a light-year a second.
#[test]
fn the_nearest_stars_are_a_minute_away() {
    for (ly, low, high) in [(1.0, 40.0, 100.0), (3.5, 45.0, 100.0), (10.0, 50.0, 120.0)] {
        let seconds = trip_seconds(ly * LY, true);
        assert!(seconds > low && seconds < high, "{ly} ly: {seconds} s");
    }
}

/// Across a galaxy the jump tier takes about ten minutes, and between galaxies the galaxy tier
/// takes about a quarter of an hour for the 3 million light-years of one grid cell.
#[test]
fn a_galaxy_takes_minutes_and_the_next_one_a_quarter_hour() {
    let across = trip_seconds(1.0e5 * LY, true);
    assert!(across > 540.0 && across < 700.0, "{across} s");
    let next = trip_seconds(GALAXY_CELL, false);
    assert!(next > 840.0 && next < 1000.0, "{next} s");
}

/// Inside a tier a longer trip never takes less time, from 0.1 light-year to 3 million, and the
/// average speed times the time is the distance. The sweep stops at the end of each tier: the
/// jump tier flies faster than the cruise tier, so a trip just over [`JUMP_DISTANCE`] may take
/// less time than one just under it.
#[test]
fn a_longer_trip_takes_longer_within_a_tier() {
    let tiers = [
        (true, 0.1 * LY, JUMP_DISTANCE),
        (true, 1.001 * JUMP_DISTANCE, 1.0e5 * LY),
        (false, 0.1 * LY, 10.0 * GALAXY_CELL),
    ];
    for (same, from, to) in tiers {
        let mut gap = from;
        let mut before = trip_seconds(gap, same);
        while gap < to {
            gap = (gap * 1.37).min(to);
            let now = trip_seconds(gap, same);
            assert!(now >= before, "{gap} m: {now} s after {before} s");
            assert!((pace(gap, same) * now - gap).abs() <= 1e-9 * gap);
            before = now;
        }
    }
    assert_eq!(trip_seconds(0.0, true), SPOOL_TIME);
    assert_eq!(trip_seconds(-5.0, true), SPOOL_TIME);
}

/// A place is in the same galaxy when it lies in the galaxy the ship is inside, and a ship
/// outside every galaxy flies the galaxy tier.
#[test]
fn the_tier_follows_the_galaxy_of_the_ship() {
    let galaxy = GalaxyId(IVec3::new(2, 0, -1));
    let other = GalaxyId(IVec3::new(5, 0, 1));
    let star = StarId {
        galaxy,
        ..Default::default()
    };
    let body = BodyId {
        star,
        ..Default::default()
    };
    let here = [None, None, Some(Target::Galaxy(galaxy))];
    assert!(same_galaxy(here, Target::Star(star)));
    assert!(same_galaxy(here, Target::Body(body)));
    assert!(same_galaxy(here, Target::Galaxy(galaxy)));
    assert!(!same_galaxy(here, Target::Galaxy(other)));
    assert!(!same_galaxy([None; 3], Target::Star(star)));
}
