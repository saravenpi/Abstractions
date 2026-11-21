//! A course to a planet of another star system: one command, and the ship arrives at that planet.

use super::{ground, key, lifts_off};
use crate::computer::{Engage, Target};
use crate::ship::tests::rig::{Game, Log};
use crate::ship::{Autopilot, Phase, ShipEvent};
use crate::space::Location;
use crate::universe::{BodyId, Universe};

/// Frames the whole course may take, at a tenth of a second each.
const FRAMES: usize = 12_000;

/// Light-years around the home star the other system is looked for in.
const NEAR: f64 = 12.0;

/// Returns the first planet of the nearest other star of the home galaxy that has planets.
fn neighbour(game: &Game) -> BodyId {
    let universe = game.get::<Universe>();
    let home = universe.home();
    let at = universe.star(home.star).expect("the home star").pos;
    let stars = universe.stars_near(home.star.galaxy, at, NEAR);
    let with_planets = |star: &&crate::universe::StarSpec| {
        star.id != home.star && !universe.system(star.id).planets.is_empty()
    };
    let star = stars.iter().find(with_planets).expect("another system");
    universe.system(star.id).planets[0].id
}

/// Takes the ship off from the pad and lets go of the levers, so that the autopilot flies it.
fn takes_off(game: &mut Game) {
    game.start("Cross", None);
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

/// Runs frames until the autopilot arrives, failing when the course is cancelled on the way or takes
/// more than [`FRAMES`].
fn flies_until_arrived(game: &mut Game, far: BodyId) {
    for frame in 0..FRAMES {
        let pilot = game.get::<Autopilot>();
        if pilot.phase() == Phase::Arrived {
            return;
        }
        let kept = pilot.target() == Some(Target::Body(far));
        assert!(kept, "the course was cancelled at frame {frame}");
        game.step(1);
    }
    panic!("the course never arrived");
}

/// One command from the air of the home planet to a planet of another system: the ship climbs, warps to
/// the other star, where the drive drops out at its lock sphere, spools again by itself, and arrives at the
/// planet, which becomes the body of the player. The course is never cancelled on the way.
#[test]
fn a_course_to_another_system_arrives_without_a_second_command() {
    let mut game = Game::full("cross");
    let game = &mut game;
    takes_off(game);
    let far = neighbour(game);
    game.app.world_mut().trigger(Engage(Target::Body(far)));
    flies_until_arrived(game, far);
    game.step(3);
    assert_eq!(game.get::<Location>().body, far);
    let log = game.get::<Log>();
    let spools = log
        .ship
        .iter()
        .filter(|event| **event == ShipEvent::WarpSpooling);
    println!(
        "voyage: the course to another system spooled {} times",
        spools.count()
    );
}
