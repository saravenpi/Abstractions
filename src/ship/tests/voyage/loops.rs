//! Courses picked on the map never loop: from the air of the home planet, from orbit above it and from
//! open space, one command takes the ship to another planet or star, the drive spools at most three times on the
//! way, and the route arrives.

use super::pick::{neighbour, other_star};
use super::place::{beside, into_low_orbit, into_orbit};
use super::{ground, key, lifts_off};
use crate::computer::{Destination, Engage, Target};
use crate::ship::tests::rig::{Game, Log};
use crate::ship::{Autopilot, Phase, ShipEvent};
use crate::space::Location;
use crate::universe::Universe;

/// Frames a course may take, at a tenth of a second each.
const FRAMES: usize = 12_000;

/// Most spools of the drive a course may take: two legs round the home planet when it lies in the way,
/// and the leg to the place. A loop spools again and again.
const SPOOLS: usize = 3;

/// Takes the ship off the pad and lets go of the levers, so that the autopilot flies it.
fn takes_off(game: &mut Game, tag: &str) {
    game.start(tag, None);
    ground::stands_on_home(game);
    ground::boards(game);
    lifts_off(game);
    for lever in [
        key(game, |keys| keys.ship.rise),
        key(game, |keys| keys.ship.throttle_up),
    ] {
        game.release(lever);
    }
}

/// Counts the spools of the drive so far.
fn spools(game: &Game) -> usize {
    let log = game.get::<Log>();
    let spooled = |event: &&ShipEvent| **event == ShipEvent::WarpSpooling;
    log.ship.iter().filter(spooled).count()
}

/// Sets the course to `target` as the Go key does, flies it, and returns the spools it took.
fn goes(game: &mut Game, target: Target) -> usize {
    let before = spools(game);
    game.app.world_mut().resource_mut::<Destination>().0 = Some(target);
    game.app.world_mut().trigger(Engage(target));
    for frame in 0..FRAMES {
        if game.get::<Autopilot>().phase() == Phase::Arrived {
            game.step(3);
            return spools(game) - before;
        }
        game.step(1);
        if frame % 2_000 == 1_999 {
            println!("loops: {:?}", game.get::<Autopilot>());
        }
    }
    let pilot = game.get::<Autopilot>();
    panic!(
        "the course never arrived after {} spools: {pilot:?}",
        spools(game) - before
    );
}

/// From the air of the home planet, one command to the nearest other planet of the system arrives with
/// at most [`SPOOLS`] spools and puts the player at that planet.
#[test]
fn a_course_from_the_air_takes_few_spools() {
    let mut game = Game::full("loops-air");
    let game = &mut game;
    takes_off(game, "Loops air");
    let far = neighbour(game);
    let spooled = goes(game, Target::Body(far));
    assert!(spooled <= SPOOLS, "{spooled} spools");
    assert_eq!(game.get::<Location>().body, far);
}

/// From orbit above the home planet, the same.
#[test]
fn a_course_from_orbit_takes_few_spools() {
    let mut game = Game::full("loops-orbit");
    let game = &mut game;
    takes_off(game, "Loops orbit");
    into_orbit(game);
    let far = neighbour(game);
    let spooled = goes(game, Target::Body(far));
    assert!(spooled <= SPOOLS, "{spooled} spools");
    assert_eq!(game.get::<Location>().body, far);
}

/// From open space, a star of another system is flown to with at most [`SPOOLS`] spools.
#[test]
fn a_course_to_a_star_takes_few_spools() {
    let mut game = Game::full("loops-star");
    let game = &mut game;
    takes_off(game, "Loops star");
    into_orbit(game);
    let spooled = goes(game, Target::Star(other_star(game)));
    assert!(spooled <= SPOOLS, "{spooled} spools");
}

/// From a circular orbit inside the lock sphere, at orbital speed, as after a climb to orbit, one command to
/// another planet climbs out of the lock first and spools at most [`SPOOLS`] times, where a spool that is
/// refused as the ship crosses the lock edge would start again and again.
#[test]
fn a_course_from_a_low_orbit_takes_few_spools() {
    let mut game = Game::full("loops-low");
    let game = &mut game;
    takes_off(game, "Loops low");
    into_low_orbit(game);
    let far = neighbour(game);
    let spooled = goes(game, Target::Body(far));
    assert!(spooled <= SPOOLS, "{spooled} spools");
    assert_eq!(game.get::<Location>().body, far);
}

/// From beside another planet of the system, a course to the home planet arrives at it with at most [`SPOOLS`]
/// spools, none of them after the last leg began to close in: the arrival point is anywhere on the shell
/// the ship crosses, so the drive is never asked again once the ship is at the planet.
#[test]
fn a_course_to_a_planet_arrives_without_a_new_spool() {
    let mut game = Game::full("loops-arrive");
    let game = &mut game;
    takes_off(game, "Loops arrive");
    let home = game.get::<Universe>().home();
    let from = neighbour(game);
    beside(game, from);
    let before = spools(game);
    let spooled = goes(game, Target::Body(home));
    assert!(spooled <= SPOOLS, "{spooled} spools");
    assert_eq!(game.get::<Location>().body, home);
    game.step(300);
    println!("loops: {:?}", game.get::<Autopilot>());
    assert_eq!(
        spools(game) - before,
        spooled,
        "the drive spooled again after the arrival"
    );
}
