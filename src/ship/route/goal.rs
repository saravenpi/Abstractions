//! Where a route ends: the lock sphere of its destination and the distance to drop out at.

use bevy::math::DVec3;

use crate::computer::Target;
use crate::ship::moons;
use crate::ship::pilot::Goal;
use crate::ship::wells::{PLANET_LOCK, STAR_LOCK};
use crate::universe::{BodyId, GLY, Universe};

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
    let lock = planet.radius * PLANET_LOCK;
    Goal {
        target: Target::Body(id),
        lock,
        arrival: lock * ARRIVAL,
        beyond: None,
    }
}

/// Returns the goal for a target: its lock sphere and where to drop out. A galaxy becomes the
/// star nearest to its centre, or its centre when no star is near.
pub(super) fn resolve(universe: &Universe, target: Target) -> Goal {
    match target {
        Target::Body(id) => body(universe, id),
        Target::Star(id) => {
            let star = universe.star(id);
            let lock = star.as_ref().map_or(GLY, |star| star.radius * STAR_LOCK);
            let calm = star.map_or(0.0, |star| (star.mu() / SAFE_PULL).sqrt());
            Goal {
                target,
                lock,
                arrival: calm.max(lock * ARRIVAL),
                beyond: None,
            }
        }
        Target::Galaxy(id) => match universe.stars_near(id, DVec3::ZERO, 30.0).first() {
            Some(star) => resolve(universe, Target::Star(star.id)),
            None => Goal {
                target,
                lock: GLY,
                arrival: GLY * ARRIVAL,
                beyond: None,
            },
        },
    }
}
