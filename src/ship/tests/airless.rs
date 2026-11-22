//! The ship where the air is missing: the assist that makes up for it, and a world without a sea.

use bevy::math::DVec3;

use super::air::{air, world};
use super::{above_pole, coast};
use crate::ship::assist::airless;
use crate::ship::clear::guard;
use crate::ship::drag::pressure;
use crate::ship::flight::Controls;
use crate::ship::landing::{GEAR_DROP, contact};
use crate::universe::PlanetRecord;
use crate::wind::samples;

/// The assist makes up for the missing air: nothing in home air or thicker, everything
/// without air, and never more as the air thickens.
#[test]
fn the_assist_takes_over_as_the_air_thins() {
    let shares = [0.0, 0.3, 1.0, 3.0].map(airless);
    assert_eq!(shares, [1.0, 0.7, 0.0, 0.0]);
    assert_eq!(pressure(0.0, 900.0), 0.0);
}

/// Returns the sideways speed left after `seconds` of full throttle over a planet of `record`
/// for a ship that slid sideways at 120 m/s, and the speed left after braking instead.
fn slide(record: PlanetRecord, assist: bool, seconds: f64) -> (f64, f64) {
    let planet = world(record);
    let run = |controls: Controls, throttle: f64| {
        let mut ship = above_pole(&planet, 30_000.0);
        (ship.vel, ship.throttle, ship.assist) = (DVec3::X * 120.0, throttle, assist);
        coast(&mut ship, &controls, (&planet, &|_| 0.0), seconds);
        ship
    };
    let brake = Controls {
        brake: true,
        ..Controls::default()
    };
    let thrust = run(Controls::default(), 1.0);
    let side = thrust.vel.dot(thrust.rot * DVec3::X).abs();
    (side, run(brake, 0.0).vel.length())
}

/// Over an airless planet the assist flies the ship as in space: under thrust the slide comes
/// onto the nose, and the brake stops the ship. With the assist off nothing is shed.
#[test]
fn the_space_assist_works_down_to_an_airless_ground() {
    let bare = samples()[5];
    let (slid, braked) = slide(bare, true, 4.0);
    assert!(slid < 1.0 && braked < 1.0, "slid {slid}, braked {braked}");
    let (free, coasting) = slide(bare, false, 4.0);
    assert!((free - 120.0).abs() < 0.1 && coasting > 120.0);
    let (thin, _) = slide(air(0.3), true, 1.0);
    let (home, _) = slide(air(1.0), true, 1.0);
    let (none, _) = slide(bare, true, 1.0);
    assert!(none < thin && thin < home, "{none} {thin} {home}");
}

/// A ship in a basin below sea level floats on a world with a sea and meets no water on a dry
/// one, where the radar reads down to the ground.
#[test]
fn a_dry_world_has_no_sea_to_float_on() {
    let basin = |_: DVec3| -20.0;
    for record in samples() {
        let planet = world(record);
        let ship = above_pole(&planet, -1.0);
        let touch = contact(&ship, &planet, &basin);
        let dry = record.ocean <= 0.0;
        assert_eq!(touch.wet, !dry, "{:?}", record.class);
        let expected = if dry { 17.75 } else { -2.25 };
        assert!((touch.radar - expected).abs() < 1e-6, "{}", touch.radar);
    }
}

/// Over the sea the turn is guarded against the water, not against the seabed far under it: a
/// level ship 2 m above the water over a seabed that climbs at 30 degrees keeps every part of
/// its stick, as it does over a flat seabed. On a dry world the same basin is ground, and its
/// slope is the plane the ship is kept clear of.
#[test]
fn over_the_sea_a_turn_is_kept_clear_of_the_water() {
    let (sea, dry) = (world(PlanetRecord::TEMPERATE), world(samples()[5]));
    let rise = 30f64.to_radians().tan();
    let seabed = |p: DVec3| -400.0 + sea.radius * p.x.atan2(p.y) * rise;
    let ship = above_pole(&sea, GEAR_DROP + 2.0);
    let touch = contact(&ship, &sea, &seabed);
    assert!((touch.radar - 2.0).abs() < 1e-6 && !touch.ground);
    assert_eq!(touch.normal, DVec3::Y);
    for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
        for stick in [axis, -axis] {
            assert_eq!(guard(&ship, &touch, stick), stick, "{stick}");
        }
    }
    let low = above_pole(&dry, GEAR_DROP + 2.0 - 400.0);
    let basin = contact(&low, &dry, &seabed);
    assert!((basin.radar - 2.0).abs() < 1e-6);
    assert!((basin.normal.dot(DVec3::Y) - 30f64.to_radians().cos()).abs() < 0.01);
    assert_ne!(guard(&low, &basin, DVec3::NEG_Z), DVec3::NEG_Z);
}
