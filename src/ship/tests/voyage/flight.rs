//! Flying the autopilot to another planet: the wait for the arrival, with a report of the flight.

use super::away::distance_to;
use crate::computer::RouteStatus;
use crate::ship::tests::rig::Game;
use crate::ship::wells::PLANET_LOCK;
use crate::ship::{Autopilot, Phase};
use crate::universe::{BodyId, Universe};

/// Frames the ship has to arrive at the other planet, at a tenth of a second each.
const FRAMES: usize = 9000;

/// Frames between two lines of the report while the ship flies.
const EVERY: usize = 1000;

/// Describes the flight to `far` now: the phase, the drive, the speed and the distance to the planet.
fn report(game: &mut Game, far: BodyId) -> String {
    let ship = game.ship();
    let phase = game.get::<Autopilot>().phase();
    let route = game.get::<RouteStatus>();
    let gap = distance_to(game, far);
    let lock = game.get::<Universe>().planet(far).radius * PLANET_LOCK;
    format!(
        "phase {phase:?}, route {route:?}, warp {:?}, speed {:.4e} m/s, {gap:.4e} m from the planet, {:.2} lock \
         radii, frame {:?}",
        ship.warp,
        ship.vel.length(),
        gap / lock,
        ship.frame
    )
}

/// Runs frames until the autopilot arrives at `far`, printing the flight every [`EVERY`] frames, and
/// fails with the report of the flight when it does not within [`FRAMES`].
pub fn until_arrived(game: &mut Game, far: BodyId) {
    for frame in 0..FRAMES {
        if game.get::<Autopilot>().phase() == Phase::Arrived {
            return;
        }
        game.step(1);
        if frame % EVERY == EVERY - 1 {
            println!("voyage: {}", report(game, far));
        }
    }
    let arrived = game.get::<Autopilot>().phase() == Phase::Arrived;
    assert!(arrived, "the ship to arrive: {}", report(game, far));
}
