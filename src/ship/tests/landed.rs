//! The assisted ship on the ground and on the sea: the sink key sets it down on a slope where it
//! stays or on the water where it floats, only a rise takes it off, and on a dry planet the
//! floor of a basin is ground to park on.

use bevy::math::DVec3;

use super::air::world;
use super::hover::{assisted, stick};
use super::{above_pole, coast, fly, planet};
use crate::ship::berth::{landing_spot, parked};
use crate::ship::flight::Controls;
use crate::ship::landing::GEAR_DROP;
use crate::wind::samples;

/// Returns the height of a slope of 20 degrees that rises toward +X around the +Y pole.
fn hillside(p: DVec3) -> f64 {
    200.0 + 6000.0 * p.x.atan2(p.y) * 20.0f64.to_radians().tan()
}

/// The sink key sets the ship on a 20 degree slope without damage, where it comes to rest
/// within 3.5 m and then does not move at all for a minute. The lever alone does not wake it:
/// only the rise key takes it off again.
#[test]
fn the_sink_key_lands_on_a_slope_and_only_a_rise_takes_off() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &hillside as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 200.0 + GEAR_DROP + 8.0);
    let (mut hit, mut touched) = (0.0f64, None);
    fly(&mut ship, &stick(-1.0), land, 20.0, |ship, report| {
        hit = hit.max(report.impact);
        touched = touched.or((report.impact > 0.0).then_some(ship.pos));
    });
    let slid = ship.pos.distance(touched.expect("a touchdown"));
    assert!(ship.landed && ship.damage == 0.0 && hit < 6.0, "hit {hit}");
    assert!(slid < 3.5, "slid {slid} m after the touchdown");
    let rested = ship.clone();
    coast(&mut ship, &Controls::default(), land, 60.0);
    ship.throttle = 0.8;
    coast(&mut ship, &Controls::default(), land, 5.0);
    assert_eq!(
        (ship.pos, ship.rot, ship.landed),
        (rested.pos, rested.rot, true)
    );
    coast(&mut ship, &stick(1.0), land, 3.0);
    assert!(!ship.landed && ship.pos.length() - rested.pos.length() > 10.0);
}

/// A ship set down on a 20 degree slope stands still on its feet before the ground holds it:
/// over the quarter second before it comes to rest it moves at under 2 cm/s. The friction of the
/// feet carries the weight along the slope, so the ship does not creep down it.
#[test]
fn the_feet_hold_on_a_slope_without_creeping() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &hillside as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 200.0 + GEAR_DROP + 8.0);
    let mut track = Vec::new();
    fly(&mut ship, &stick(-1.0), land, 20.0, |ship, _| {
        track.push((ship.vel.length(), ship.landed));
    });
    let rested = track.iter().position(|step| step.1).expect("a rest");
    let before = &track[rested.saturating_sub(15)..rested];
    let creep = before.iter().map(|step| step.0).fold(0.0, f64::max);
    assert!(creep < 0.02, "the ship creeps at {creep} m/s before it rests");
}

/// On a planet without a sea the ship starts on the floor of a basin below sea level and
/// rests there. On a planet with a sea it starts on dry land beside the same basin.
#[test]
fn a_dry_basin_is_ground_to_park_on() {
    let planet = world(samples()[5]);
    let near = DVec3::new(0.3, 0.9, -0.2).normalize() * planet.radius;
    let basin = |p: DVec3| match (p.normalize() * planet.radius).distance(near) {
        from if from < 40.0 => -20.0,
        from => 5.0 + 0.02 * from,
    };
    let dry = landing_spot(&basin, near, f64::NEG_INFINITY);
    let wet = landing_spot(&basin, near, 0.0);
    assert!(dry.distance(near) < 20.0 && wet.distance(near) > 44.0);
    let ship = parked(planet.id, (&basin, f64::NEG_INFINITY), dry, planet.radius);
    assert!((ship.pos.length() - planet.radius - GEAR_DROP + 20.0).abs() < 1e-6);
    let afloat = parked(planet.id, (&basin, 0.0), dry, planet.radius);
    assert!((afloat.pos.length() - planet.radius - GEAR_DROP).abs() < 1e-6);
    let mut dropped = above_pole(&planet, GEAR_DROP - 19.7);
    coast(
        &mut dropped,
        &Controls::default(),
        (&planet, &|_| -20.0),
        6.0,
    );
    let height = dropped.pos.length() - planet.radius;
    assert!(
        dropped.landed && (height - GEAR_DROP + 20.0).abs() < 0.3,
        "{height}"
    );
}

/// Over deep water the sink key sets the ship on the sea, where it floats and comes to rest
/// without damage, and stays at rest when the key is released.
#[test]
fn the_sink_key_sets_the_ship_on_the_sea() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let land = (&planet, &(|_| -80.0) as &dyn Fn(DVec3) -> f64);
    let mut ship = assisted(&planet, 8.0);
    coast(&mut ship, &stick(-1.0), land, 30.0);
    let height = ship.pos.length() - planet.radius;
    assert!(ship.landed && ship.damage == 0.0, "afloat at {height}");
    assert!(height > 0.0 && height < GEAR_DROP, "floating at {height}");
    let rested = ship.pos;
    coast(&mut ship, &Controls::default(), land, 10.0);
    assert!(ship.landed && ship.pos == rested);
}
