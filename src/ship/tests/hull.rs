//! The hull against what is built: a ship flown at a house wall stops at it, slow or fast, and a
//! ship lowered onto a flat roof or onto the canopy of the base rests on it. Until 0.10 the ship
//! read only the terrain height and flew through all of them.

mod stage;

use bevy::math::DVec3;
use bevy::prelude::*;

use super::hover::{assisted, stick};
use super::{above_pole, planet};
use crate::ship::body::Ship;
use crate::ship::flight::{Controls, Report};
use crate::ship::landing::GEAR_DROP;
use crate::universe::PlanetSpec;
use stage::{Block, CANOPY, CANOPY_TOP, flown, stage};

/// A house 8 m wide and 5 m high, its front wall at z = -10 m, the ship coming at it along -z.
const HOUSE: [Block; 1] = [(Vec3::new(0.0, 2.5, -14.0), Vec3::new(8.0, 5.0, 8.0))];

/// Where the front wall of [`HOUSE`] is along z, in metres.
const WALL: f64 = -10.0;

/// Metres from the hull centre to the tip of the nose.
const NOSE: f64 = 4.2;

/// A house with a flat roof 10 m on a side, its top 4 m above the sea.
const ROOF: [Block; 1] = [(Vec3::new(0.0, 2.0, 0.0), Vec3::new(10.0, 4.0, 10.0))];

/// Height in metres above the sea of the top of [`ROOF`].
const ROOF_TOP: f64 = 4.0;

/// Returns the home planet without gravity, air or spin, so that a ship let go coasts straight.
fn still() -> PlanetSpec {
    PlanetSpec {
        gravity: 0.0,
        atmosphere_height: 0.0,
        ..planet(6000.0, 4000.0, 1e12)
    }
}

/// Returns the height in metres above the sea of the hull centre of `ship` over `planet`.
fn height(ship: &Ship, planet: &PlanetSpec) -> f64 {
    ship.pos.length() - planet.radius
}

/// Flies a ship without assist level at [`HOUSE`] at `speed` metres per second for two seconds.
fn into_wall(speed: f64) -> Vec<(Ship, Report)> {
    let planet = still();
    let mut ship = above_pole(&planet, 2.5);
    ship.vel = DVec3::NEG_Z * speed;
    let mut app = stage(&HOUSE);
    flown(&mut app, ship, Controls::default(), (planet, -50.0), 2.0)
}

/// Checks that the nose of the ship never passed the wall, and that the ship ended at the wall,
/// stopped against it.
fn stopped_at_the_wall(track: &[(Ship, Report)]) {
    let deepest = track
        .iter()
        .map(|(ship, _)| ship.pos.z - NOSE)
        .fold(0.0, f64::min);
    assert!(deepest > WALL - 0.05, "the nose went to z = {deepest}");
    let (last, _) = track.last().expect("a flight");
    assert!(
        last.pos.z - NOSE < WALL + 0.3,
        "the ship stopped at z = {}",
        last.pos.z
    );
    assert!(last.vel.z.abs() < 0.01, "still at {} m/s", last.vel.z);
}

/// A ship that drifts into a house wall at 5 m/s stops at the wall without passing through it,
/// and the bump is no crash.
#[test]
#[ignore = "fails until the hull meets what is built: the ship flies through"]
fn a_ship_stops_at_a_house_wall_at_5_m_s() {
    let track = into_wall(5.0);
    stopped_at_the_wall(&track);
    assert!(track.iter().all(|(_, report)| report.crash.is_none()));
}

/// A ship that flies into a house wall at 60 m/s stops at the wall without passing through it,
/// takes damage and reports the crash.
#[test]
#[ignore = "fails until the hull meets what is built: the ship flies through"]
fn a_ship_stops_at_a_house_wall_at_60_m_s() {
    let track = into_wall(60.0);
    stopped_at_the_wall(&track);
    let (last, _) = track.last().expect("a flight");
    assert!(last.damage > 0.5, "damage {}", last.damage);
    assert!(track.iter().any(|(_, report)| report.crash.is_some()));
}

/// Lowers an assisted ship with the sink key from `above` metres over the sea onto `blocks`,
/// for twenty seconds.
fn lowered(blocks: &[Block], above: f64) -> (PlanetSpec, Vec<(Ship, Report)>) {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let ship = assisted(&planet, above);
    let mut app = stage(blocks);
    let track = flown(&mut app, ship, stick(-1.0), (planet.clone(), 0.0), 20.0);
    (planet, track)
}

/// The sink key sets the ship down on a flat roof, where it rests on its gear without damage.
#[test]
#[ignore = "fails until the hull meets what is built: the ship flies through"]
fn a_ship_lowered_onto_a_flat_roof_rests_on_it() {
    let (planet, track) = lowered(&ROOF, ROOF_TOP + GEAR_DROP + 8.0);
    let lowest = track
        .iter()
        .map(|(ship, _)| height(ship, &planet))
        .fold(f64::INFINITY, f64::min);
    let (last, _) = track.last().expect("a flight");
    let rest = height(last, &planet) - ROOF_TOP - GEAR_DROP;
    assert!(
        last.landed && last.damage == 0.0,
        "landed {} damage {}",
        last.landed,
        last.damage
    );
    assert!(rest.abs() < 0.3, "rests {rest} m off the roof");
    assert!(lowest > ROOF_TOP + GEAR_DROP - 0.3, "went down to {lowest}");
}

/// The sink key sets the ship down on the canopy of the base, where it rests without damage and
/// without its belly ever going under the top of the canopy.
#[test]
#[ignore = "fails until the hull meets what is built: the ship flies through"]
fn a_ship_lowered_onto_the_base_canopy_rests_on_it() {
    let (planet, track) = lowered(&CANOPY, CANOPY_TOP + GEAR_DROP + 6.0);
    let lowest = track
        .iter()
        .map(|(ship, _)| height(ship, &planet))
        .fold(f64::INFINITY, f64::min);
    let (last, _) = track.last().expect("a flight");
    assert!(
        last.landed && last.damage == 0.0,
        "landed {} damage {}",
        last.landed,
        last.damage
    );
    assert!(
        lowest > CANOPY_TOP + 0.6,
        "the hull centre went down to {lowest}"
    );
}

/// A ship that falls at 40 m/s onto the canopy, a slab 0.1 m thick, does not pass through it.
#[test]
#[ignore = "fails until the hull meets what is built: the ship flies through"]
fn a_ship_dropped_fast_on_the_canopy_does_not_pass_through() {
    let planet = planet(6000.0, 4000.0, 1200.0);
    let mut ship = above_pole(&planet, CANOPY_TOP + 10.0);
    ship.vel = DVec3::NEG_Y * 40.0;
    let mut app = stage(&CANOPY);
    let track = flown(
        &mut app,
        ship,
        Controls::default(),
        (planet.clone(), 0.0),
        3.0,
    );
    let lowest = track
        .iter()
        .map(|(ship, _)| height(ship, &planet))
        .fold(f64::INFINITY, f64::min);
    assert!(
        lowest > CANOPY_TOP + 0.5,
        "the hull centre went down to {lowest}"
    );
    assert!(track.iter().any(|(_, report)| report.crash.is_some()));
}
