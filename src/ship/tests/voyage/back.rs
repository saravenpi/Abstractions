//! The way back from a save: the same place on foot, and the same flight in the seat.

use super::key;
use crate::audio::ChirpKind;
use crate::companion::CompanionChirp;
use crate::computer::{Atlas, Target};
use crate::saves::WorldEntry;
use crate::ship::tests::rig::{FLIGHT, Game, Log};
use crate::ship::{Ship, ShipState, ShipTelemetry};
use crate::space::{Clock, Frame, Location, Travel};
use crate::universe::BodyId;
use crate::world::Planet;

/// Returns the ship record a world file holds.
fn record(entry: &WorldEntry) -> ShipState {
    let ship = entry.save.ship.clone().expect("the save holds the ship");
    ship.try_into().expect("the ship record reads")
}

/// Asserts that two ships are the same ship at the same place: frame, pose, velocity and rest.
fn assert_same(a: &Ship, b: &Ship) {
    assert_eq!((a.frame, a.landed), (b.frame, b.landed));
    assert!(
        a.pos.distance(b.pos) < 1e-9,
        "moved {}",
        a.pos.distance(b.pos)
    );
    assert!(a.rot.angle_between(b.rot) < 1e-9 && a.vel.distance(b.vel) < 1e-9);
    assert_eq!(
        (a.throttle, a.damage, a.assist),
        (b.throttle, b.damage, b.assist)
    );
}

/// Saves, quits to the title screen and loads the file again. Returns what the file held.
fn reload(game: &mut Game) -> WorldEntry {
    let entry = game.save_and_quit();
    assert!(game.app.world().get_resource::<Planet>().is_none());
    game.start("", Some(entry.clone()));
    entry
}

/// Checks that the loaded world is at planet `far` with everything the atlas knew, at the
/// universe time of the save.
fn at_planet(game: &mut Game, entry: &WorldEntry, far: BodyId) {
    assert_eq!(
        entry.save.body(&game.get::<crate::universe::Universe>()),
        far
    );
    assert_eq!(game.get::<Location>().body, far);
    assert_eq!(game.get::<Planet>().body, far);
    let atlas = game.get::<Atlas>();
    assert!(atlas.visited(Target::Body(far)) && atlas.walked.contains(&far));
    let passed = game.get::<Clock>().seconds - entry.save.clock_seconds;
    assert!(
        (0.0..1.0).contains(&passed),
        "universe time moved by {passed} s"
    );
}

/// A save taken on foot on the new planet loads to the same spot, beside the same landed ship,
/// and the companion does not celebrate the planet a second time.
pub fn resumes_on_foot(game: &mut Game, far: BodyId) {
    let (stood, parked) = (game.walker(), game.ship());
    let entry = reload(game);
    at_planet(game, &entry, far);
    assert_eq!(game.get::<Location>().travel, Travel::OnFoot);
    assert_same(&game.ship(), &parked);
    let read = Ship::from_state(&record(&entry)).expect("a sound record");
    assert_same(&read, &parked);
    game.until("the player to stand", 300, |game| game.walker().grounded);
    game.step(30);
    let walker = game.walker();
    assert_eq!(walker.body, Some(far));
    let moved = walker.at.distance(stood.at);
    assert!(moved < 0.3, "the player came back {moved} m away");
    assert!(walker.gap.abs() < 0.6 && !walker.seated);
    let chirps = game.get::<Log>().chirps;
    let delights = |kind: &&CompanionChirp| kind.0 == ChirpKind::Delight;
    assert_eq!(chirps.iter().filter(delights).count(), 1);
}

/// A save taken in the seat of the flying ship loads seated, in the same flight.
pub fn resumes_in_flight(game: &mut Game, far: BodyId) {
    assert!(game.get::<crate::ship::Hatch>().near);
    game.press(key(game, |keys| keys.ship.board));
    game.step(1);
    game.release(key(game, |keys| keys.ship.board));
    game.press(key(game, |keys| keys.ship.rise));
    game.pace(FLIGHT);
    game.until("the ship to climb", 600, |game| {
        game.get::<ShipTelemetry>().altitude_ground > 25.0
    });
    let entry = reload(game);
    game.release(key(game, |keys| keys.ship.rise));
    at_planet(game, &entry, far);
    let (saved, ship) = (record(&entry), game.ship());
    assert!(saved.seated && !saved.landed);
    assert_eq!(saved.frame, Frame::Surface(far));
    assert_eq!(game.get::<Location>().travel, Travel::InShip);
    assert!(game.walker().seated, "the body is not in the seat");
    let flown = (game.get::<Clock>().seconds - entry.save.clock_seconds).max(FLIGHT);
    let drift = ship.pos.distance(saved.pos.into());
    let speed = bevy::math::DVec3::from(saved.vel).length() + 60.0 * flown;
    assert!(flown < 1.0, "{flown} s passed during the load");
    assert!(
        drift <= speed * flown + 0.01,
        "the ship jumped by {drift} m"
    );
    assert!(!ship.landed && ship.frame == Frame::Surface(far));
}
