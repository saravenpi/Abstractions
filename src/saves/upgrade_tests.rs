//! Save tests for files from 0.8.1: one on the home planet starts again at the new base with
//! what it discovered, one on another planet keeps its place on the new ground.

use bevy::math::{DQuat, DVec3};

use super::model::{Place, SAVE_VERSION, WorldSave};
use crate::computer::{Atlas, Target};
use crate::ship::{Ship, ShipState};
use crate::sim::{Change, Changes, PlayerId};
use crate::space::Frame;
use crate::universe::{BodyId, Universe};
use crate::world::Globe;

/// Seed of the worlds of these tests.
const SEED: u64 = 7;

/// Returns the first body after the home planet in its system that has ground.
fn elsewhere(universe: &Universe) -> BodyId {
    let home = universe.home();
    let others = (0..12).filter(|index| *index != home.index);
    let mut bodies = others.map(|index| BodyId { index, ..home });
    let ground = |body: &BodyId| Globe::of(&universe.planet(*body)).has_ground();
    bodies.find(ground).expect("a second body with ground")
}

/// Returns a file as 0.8.1 wrote it: version 1, standing `altitude` metres above sea level
/// near the pole of `body` with the ship landed beside it, an atlas that visited `body`, one
/// logged change and some play time.
fn old(body: BodyId, altitude: f64) -> WorldSave {
    let universe = Universe::new(SEED);
    let mut atlas = Atlas::new(&universe);
    atlas.visit(Target::Body(body));
    let mut changes = Changes::default();
    changes.push(PlayerId::LOCAL, 12.5, Change::Visited(body));
    let radius = universe.planet(body).radius;
    let dir = DVec3::new(0.02, 1.0, 0.01).normalize();
    let at = dir * (radius + altitude + 0.35);
    let mut ship = Ship::at_rest(Frame::Surface(body), at, DQuat::IDENTITY);
    ship.landed = true;
    WorldSave {
        seed: SEED,
        version: 1,
        yaw: 0.7,
        play_seconds: 4321.0,
        clock_seconds: 9876.5,
        place: Some(Place {
            body,
            dir: dir.to_array(),
            altitude,
        }),
        atlas: Some(atlas),
        ship: Some(toml::Value::try_from(ship.state()).unwrap()),
        changes,
        ..WorldSave::default()
    }
}

/// Returns the record of the ship of `save`.
fn ship(save: &WorldSave) -> ShipState {
    save.ship.clone().unwrap().try_into().unwrap()
}

/// A 0.8.1 file on the home planet begins again as a new game does, at the new base with a
/// fresh ship, and keeps its atlas, its log, its play time and its universe time.
#[test]
fn a_081_save_on_the_home_planet_starts_at_the_new_base() {
    let universe = Universe::new(SEED);
    let before = old(universe.home(), 3.0);
    let mut save = before.clone();
    save.upgrade();
    assert_eq!(save.version, SAVE_VERSION);
    assert_eq!((save.place, save.position, &save.ship), (None, None, &None));
    assert_eq!(save.body(&universe), universe.home());
    assert_eq!(save.atlas, before.atlas);
    assert_eq!(save.changes, before.changes);
    assert_eq!(
        (save.play_seconds, save.clock_seconds),
        (before.play_seconds, before.clock_seconds)
    );
    let mut again = save.clone();
    again.upgrade();
    assert_eq!(again, save);
}

/// A ship left on the home planet takes the file home too, and a player seated in a ship that
/// flies somewhere else only last stood at home: that file keeps its ship.
#[test]
fn the_ship_decides_for_a_player_in_its_seat() {
    let universe = Universe::new(SEED);
    let (home, other) = (universe.home(), elsewhere(&universe));
    let mut docked = old(home, 3.0);
    docked.place = old(other, 3.0).place;
    docked.upgrade();
    assert_eq!((docked.place, &docked.ship), (None, &None));
    let mut flying = old(home, 3.0);
    let seat = ShipState {
        seated: true,
        ..ship(&old(other, 9000.0))
    };
    flying.ship = Some(toml::Value::try_from(seat).unwrap());
    flying.upgrade();
    assert_eq!(flying.place.unwrap().body, home);
    assert_eq!(Frame::from(ship(&flying).frame), Frame::Surface(other));
}

/// A 0.8.1 file on another planet keeps its place and its ship there, and both stand on the
/// ground of 0.9: a body saved under it is lifted onto it, one above it is left where it was.
#[test]
fn a_081_save_on_another_planet_keeps_its_place() {
    let universe = Universe::new(SEED);
    let other = elsewhere(&universe);
    let globe = Globe::of(&universe.planet(other));
    let before = old(other, -9000.0);
    let dir = DVec3::from(before.place.unwrap().dir);
    let ground = f64::from(globe.height(dir));
    let mut sunk = before.clone();
    sunk.upgrade();
    let place = sunk.place.unwrap();
    assert_eq!(
        (sunk.version, place.body, place.dir),
        (2, other, dir.to_array())
    );
    assert!((place.altitude - ground - 0.9).abs() < 1.0e-3, "{place:?}");
    assert_eq!((&sunk.atlas, sunk.yaw), (&before.atlas, before.yaw));
    let hull = DVec3::from_array(ship(&sunk).pos);
    let top = f64::from(globe.height(hull.normalize()).max(globe.sea_level()));
    assert!((hull.length() - globe.radius() - top - 1.25).abs() < 1.0e-3);
    assert!(ship(&sunk).landed);
    let mut high = old(other, ground + 500.0);
    high.upgrade();
    assert_eq!(high.place.unwrap().altitude, ground + 500.0);
}
