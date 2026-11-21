//! A course to another galaxy, end to end: one command, a few cycles of the drive, an arrival, and a save in
//! the middle of the course that resumes and completes.

use super::{ground, key, lifts_off};
use crate::computer::{Destination, Engage, RouteStatus, Target, ship_position, target_position};
use crate::ship::tests::rig::{Game, Log};
use crate::ship::{Autopilot, Phase, ShipEvent};
use crate::space::{Clock, Location};
use crate::universe::{GalaxyId, Universe};

/// Frames the whole course may take, at a tenth of a second each.
const FRAMES: usize = 40_000;

/// Most times the drive may spool on the whole course: the climb out of the system and the legs round a
/// star, never a loop.
const SPOOLS: usize = 6;

/// Frames of the course flown before the save of the resume test.
const BEFORE_SAVE: usize = 1_500;

/// Returns the nearest galaxy that is not the home one.
fn other(game: &Game) -> GalaxyId {
    let universe = game.get::<Universe>();
    let home = universe.home().star.galaxy;
    let galaxies = universe.galaxies_near(home, 1);
    galaxies
        .iter()
        .find(|galaxy| galaxy.id != home)
        .expect("another galaxy")
        .id
}

/// Takes off from the pad and sends the ship to the other galaxy with one command.
fn sets_out(game: &mut Game, tag: &str) -> GalaxyId {
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
    let far = other(game);
    game.app.world_mut().resource_mut::<Destination>().0 = Some(Target::Galaxy(far));
    game.app.world_mut().trigger(Engage(Target::Galaxy(far)));
    far
}

/// Frames between two lines of the report while the ship flies.
const EVERY: usize = 2_000;

/// Describes the flight now: the route and its guard, the drive, the frame, the metres left to the
/// centre of the galaxy and the spools so far.
fn report(game: &mut Game, far: GalaxyId) -> String {
    let (ship, pilot) = (game.ship(), game.get::<Autopilot>());
    let (universe, time) = (game.get::<Universe>(), game.get::<Clock>().seconds);
    let here = ship_position(&universe, ship.frame, ship.pos, time);
    let there = target_position(&universe, Target::Galaxy(far), time);
    let left = here.zip(there).map(|(here, there)| here.distance(there));
    let route = game.get::<RouteStatus>();
    let spooled = spools(game);
    format!(
        "{pilot:?}, route {route:?}, warp {:?}, frame {:?}, {left:?} m left, {spooled} spools",
        ship.warp, ship.frame
    )
}

/// Runs frames until the autopilot arrives, within [`FRAMES`], printing the flight every [`EVERY`] frames,
/// and returns how many frames it took. It fails with the report of the flight otherwise.
fn flies(game: &mut Game, far: GalaxyId, from: usize) -> usize {
    for frame in from..FRAMES {
        if game.get::<Autopilot>().phase() == Phase::Arrived {
            return frame;
        }
        game.step(1);
        if frame % EVERY == EVERY - 1 {
            println!("voyage: {}", report(game, far));
        }
    }
    panic!(
        "the course to the other galaxy never arrived: {}",
        report(game, far)
    );
}

/// Counts the times the drive spooled up so far.
fn spools(game: &Game) -> usize {
    let log = game.get::<Log>();
    log.ship
        .iter()
        .filter(|event| **event == ShipEvent::WarpSpooling)
        .count()
}

/// One command to another galaxy: the ship arrives in its frame, and the drive spooled up only a few
/// times on the way, where a loop would spool it again and again.
#[test]
fn a_course_to_another_galaxy_arrives_in_a_few_cycles() {
    let mut game = Game::full("galaxy");
    let game = &mut game;
    let far = sets_out(game, "Galaxy");
    flies(game, far, 0);
    game.step(3);
    assert_eq!(game.get::<Location>().body.star.galaxy, far);
    let cycles = spools(game);
    println!("voyage: the course to another galaxy spooled {cycles} times");
    assert!((1..=SPOOLS).contains(&cycles), "{cycles} spools");
}

/// A save taken in the middle of the course keeps the course, and the course set again after the load flies
/// the rest of the way and arrives.
#[test]
fn a_save_in_the_middle_of_a_galaxy_course_resumes_and_arrives() {
    let mut game = Game::full("galaxy-save");
    let game = &mut game;
    let far = sets_out(game, "Galaxy save");
    game.step(BEFORE_SAVE);
    let course = game.get::<Destination>().0;
    assert_eq!(course, Some(Target::Galaxy(far)));
    let entry = game.save_and_quit();
    game.start("", Some(entry));
    assert_eq!(game.get::<Destination>().0, course);
    game.app.world_mut().trigger(Engage(Target::Galaxy(far)));
    let frames = flies(game, far, 0);
    game.step(3);
    assert_eq!(game.get::<Location>().body.star.galaxy, far);
    println!("voyage: the resumed course arrived after {frames} frames");
}
