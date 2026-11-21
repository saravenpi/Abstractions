//! The course of a route: its speed tier and the point to aim at.

use bevy::math::DVec3;

use super::align;
use super::goal::between_galaxies;
use crate::ship::body::Ship;
use crate::ship::pilot::Goal;
use crate::ship::warp::{Tier, Warp};
use crate::space::UniPos;
use crate::universe::Universe;

/// Returns the speed tier of the route and the course to the destination as it will be when the
/// ship gets there, so a destination that moves is led and the heading needs no correction on the
/// way. The tier is set while the drive is off from the length of the route and whether it ends
/// in another galaxy, and `held` is kept while the drive spools or runs, so the cap never changes
/// under a ship in warp. The ship goes on with its own velocity until the drive carries it, so the
/// course is the one from where it will be then. `now` is where the ship is and when, `to` the
/// course now.
pub(super) fn plan(
    goal: &Goal,
    universe: &Universe,
    now: (Option<UniPos>, f64),
    ship: &Ship,
    (to, held): (DVec3, Tier),
) -> (Tier, DVec3) {
    let tier = match ship.warp {
        Warp::Off => Tier::of(
            to.length(),
            between_galaxies(universe, ship.frame, (goal.target, to.length())),
        ),
        _ => held,
    };
    let drift = ship.vel * align::spool(ship.warp);
    let at = |here: UniPos, ahead: f64| {
        let reached = goal.reached(universe, here + drift, now.1 + ahead);
        reached.map(|(to, _)| to)
    };
    let led = now
        .0
        .and_then(|here| align::led(ship.warp, (goal.lock, tier.cap()), |ahead| at(here, ahead)));
    (tier, led.unwrap_or(to))
}
