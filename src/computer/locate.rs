//! Where the ship and its targets are in the universe.

use bevy::math::DVec3;

use super::route::Target;
use crate::space::{Frame, UniPos};
use crate::universe::{BodyId, GLY, GalaxyId, StarId, Universe};

/// Distance from a star in metres within which the ship counts as inside its system.
pub const SYSTEM_REACH: f64 = 0.1 * GLY;

/// Returns the universe position of the point `at` of `frame` at `time` seconds.
///
/// On a planet the spin of the planet is ignored, which moves the result by less than its diameter.
pub fn ship_position(universe: &Universe, frame: Frame, at: DVec3, time: f64) -> Option<UniPos> {
    let (star, offset) = match frame {
        Frame::Surface(body) => (body.star, universe.planet_position(body, time) + at),
        Frame::System(star) => (star, at),
        Frame::Galaxy(id) => return Some(universe.galaxy(id)?.locate(at)),
    };
    Some(universe.star_position(star)? + offset)
}

/// Returns the universe position of the centre of `target` at `time` seconds.
pub fn target_position(universe: &Universe, target: Target, time: f64) -> Option<UniPos> {
    match target {
        Target::Body(id) => {
            Some(universe.star_position(id.star)? + universe.planet_position(id, time))
        }
        Target::Star(id) => universe.star_position(id),
        Target::Galaxy(id) => Some(universe.galaxy(id)?.locate(DVec3::ZERO)),
    }
}

/// Returns the planet, system and galaxy that the point `at` of `frame` lies inside.
pub fn whereabouts(
    universe: &Universe,
    frame: Frame,
    at: DVec3,
) -> (Option<BodyId>, Option<StarId>, Option<GalaxyId>) {
    match frame {
        Frame::Surface(body) => (Some(body), Some(body.star), Some(body.star.galaxy)),
        Frame::System(star) => {
            let inside = at.length() <= SYSTEM_REACH;
            (None, inside.then_some(star), Some(star.galaxy))
        }
        Frame::Galaxy(id) => {
            let galaxy = universe.galaxy(id);
            let inside = galaxy.is_some_and(|galaxy| at.length() <= galaxy.radius);
            (None, None, inside.then_some(id))
        }
    }
}
