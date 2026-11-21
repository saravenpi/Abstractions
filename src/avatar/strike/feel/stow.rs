//! What the hands ask of the rest of the robot: a drawn tool squares the body to the view, and
//! opening the wingsuit or boarding the ship slings the held tool over the back first. A slung tool
//! stays slung after the landing until the player presses its slot.

use bevy::prelude::*;

use super::super::act::{Act, Strike};
use super::super::held::Held;
use super::super::input::Robot;
use crate::gear::{Gear, Mode as Flight};
use crate::player::{Motion, Player, Squared};
use crate::space::{Location, Travel};

/// Returns true when a tool that is `out` doing `act` is to be slung now: the wingsuit is asked
/// open or the robot boarded, and it is not being slung already.
pub fn stows(out: bool, act: Act, (wings, boarded): (bool, bool)) -> bool {
    out && (wings || boarded) && !matches!(act, Act::Stow(_))
}

/// Slings the tool of the local robot over its back as the wingsuit opens or the robot boards.
pub fn stow(
    robot: Robot,
    players: Query<(&Motion, &Gear), With<Player>>,
    location: Option<Res<Location>>,
    mut bodies: Query<(&mut Strike, &Held)>,
) {
    let Some((body, player)) = robot.pair() else {
        return;
    };
    let (Ok((mut strike, held)), Ok((motion, gear))) = (bodies.get_mut(body), players.get(player))
    else {
        return;
    };
    let wings = motion.wings || gear.state.mode == Flight::Glide;
    let boarded = location.is_some_and(|at| at.travel == Travel::InShip);
    if stows(held.out, strike.act, (wings, boarded)) {
        strike.act = Act::Stow(0.0);
    }
}

/// Run condition: some body holds a tool, or [`Squared`] still needs clearing.
pub fn squaring(held: Query<(), With<Held>>, squared: Res<Squared>) -> bool {
    !held.is_empty() || *squared != Squared::default()
}

/// Tells the controller whether the local robot has a tool drawn and strikes with it, so the body
/// squares to the view.
pub fn square(robot: Robot, bodies: Query<(&Strike, &Held)>, mut squared: ResMut<Squared>) {
    let hands = robot.body().and_then(|body| bodies.get(body).ok());
    let next = hands.map_or(Squared::default(), |(strike, held)| Squared {
        drawn: held.out,
        striking: held.out && strike.act != Act::Ready,
    });
    squared.set_if_neq(next);
}
