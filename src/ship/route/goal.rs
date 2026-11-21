//! Where a route ends: the lock sphere of its destination and the distance to drop out at.

use bevy::math::DVec3;

use super::shell;
use crate::computer::Target;
use crate::ship::moons;
use crate::ship::pilot::Goal;
use crate::ship::wells::STAR_LOCK;
use crate::space::Frame;
use crate::universe::{BodyId, GalaxyId, LY, StarSpec, Universe};

/// Lock radii from a planet at which the autopilot drops out, a safe distance for an orbit.
pub(in crate::ship) const ARRIVAL: f64 = 1.3;

/// Pull of a star in metres per second squared at the distance where the autopilot drops out.
const SAFE_PULL: f64 = 0.5;

/// Returns the goal for a planet, dropped out at a safe distance from its centre, or for a
/// moon, left beside it.
fn body(universe: &Universe, id: BodyId) -> Goal {
    let planet = universe.planet(id);
    if id.is_moon() {
        return moons::goal(&planet, &universe.planet(id.planet()));
    }
    Goal {
        target: Target::Body(id),
        lock: shell::lock(&planet),
        arrival: shell::radius(&planet),
        beyond: None,
    }
}

/// Shares of the radius of a galaxy at which the stars of its centre are looked for, in the
/// plane of its disc: the very centre is a void a few hundred light-years wide, and the stars
/// begin just outside it.
const CENTRE_SHARES: [f64; 3] = [0.03, 0.05, 0.08];

/// Light-years around each point of [`CENTRE_SHARES`] in which a star is looked for: 125 sectors
/// of a galaxy, 375 at most for the three points, and one lookup per route.
const CENTRE_SCAN: f64 = 20.0;

/// Returns a star at the edge of the void at the centre of galaxy `id`, from the first of a few
/// probes in its disc that finds one, or nothing when none does.
fn centre_star(universe: &Universe, id: GalaxyId) -> Option<StarSpec> {
    let radius = universe.galaxy(id)?.radius;
    let near = |share: f64| universe.stars_near(id, DVec3::X * radius * share, CENTRE_SCAN);
    CENTRE_SHARES
        .into_iter()
        .find_map(|share| near(share).into_iter().next())
}

/// Returns the galaxy `target` lies in.
fn galaxy_of(target: Target) -> GalaxyId {
    match target {
        Target::Body(body) => body.star.galaxy,
        Target::Star(star) => star.galaxy,
        Target::Galaxy(galaxy) => galaxy,
    }
}

/// Returns whether `target` lies in another galaxy than the one the frame of a ship is in: the
/// route is then a jump between galaxies.
pub(super) fn abroad(frame: Frame, target: Target) -> bool {
    let ship = match frame {
        Frame::Surface(body) => body.star.galaxy,
        Frame::System(star) => star.galaxy,
        Frame::Galaxy(galaxy) => galaxy,
    };
    ship != galaxy_of(target)
}

/// Returns whether a ship in `frame`, `gap` metres from `target`, is still between the galaxies: its
/// frame is another galaxy than that of the target, or it is the frame of the target galaxy but farther
/// from the target than the galaxy is wide. A ship enters the frame of a galaxy at the edge of its grid
/// cell, millions of light-years before the galaxy itself, and flies the galaxy tier until then.
pub(super) fn between_galaxies(
    universe: &Universe,
    frame: Frame,
    (target, gap): (Target, f64),
) -> bool {
    let wide = universe
        .galaxy(galaxy_of(target))
        .is_none_or(|galaxy| gap > 2.0 * galaxy.radius);
    abroad(frame, target) || wide
}

/// Returns the goal for a target: its lock sphere and where to drop out. A galaxy becomes a star
/// at the edge of the void at its centre, or its centre when no star is there: a ship cannot
/// stop at a point of empty space it was never shown, so that one is a lock sphere of one
/// light-year.
pub(super) fn resolve(universe: &Universe, target: Target) -> Goal {
    match target {
        Target::Body(id) => body(universe, id),
        Target::Star(id) => {
            let star = universe.star(id);
            let lock = star.as_ref().map_or(LY, |star| star.radius * STAR_LOCK);
            let calm = star.map_or(0.0, |star| (star.mu() / SAFE_PULL).sqrt());
            Goal {
                target,
                lock,
                arrival: calm.max(lock * ARRIVAL),
                beyond: None,
            }
        }
        Target::Galaxy(id) => match centre_star(universe, id) {
            Some(star) => resolve(universe, Target::Star(star.id)),
            None => Goal {
                target,
                lock: LY,
                arrival: LY * ARRIVAL,
                beyond: None,
            },
        },
    }
}
