//! Where the ship and its targets are in the universe.

use bevy::math::DVec3;

use super::route::Target;
use crate::space::{Frame, UniPos};
use crate::universe::{BodyId, GalaxyId, StarId, Universe, system_reach};

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
///
/// A ship is inside a system within [`system_reach`] of its star, the one reach the sky, the
/// ship and the map share. Reading the star is one sector fill, at most 12 stars.
pub fn whereabouts(
    universe: &Universe,
    frame: Frame,
    at: DVec3,
) -> (Option<BodyId>, Option<StarId>, Option<GalaxyId>) {
    whereabouts_with(universe, frame, at, &mut None)
}

/// Returns what [`whereabouts`] returns, keeping in `memo` the reach of the last star it read,
/// so a ship that stays in one system reads its star once and not every frame.
pub fn whereabouts_with(
    universe: &Universe,
    frame: Frame,
    at: DVec3,
    memo: &mut Option<(StarId, f64)>,
) -> (Option<BodyId>, Option<StarId>, Option<GalaxyId>) {
    match frame {
        Frame::Surface(body) => (Some(body), Some(body.star), Some(body.star.galaxy)),
        Frame::System(star) => {
            let reach = match *memo {
                Some((id, reach)) if id == star => Some(reach),
                _ => universe.star(star).map(|spec| system_reach(&spec)),
            };
            *memo = reach.map(|reach| (star, reach));
            let inside = reach.is_some_and(|reach| at.length() <= reach);
            (None, inside.then_some(star), Some(star.galaxy))
        }
        Frame::Galaxy(id) => {
            let galaxy = universe.galaxy(id);
            let inside = galaxy.is_some_and(|galaxy| at.length() <= galaxy.radius);
            (None, None, inside.then_some(id))
        }
    }
}

/// Returns the planet, the system and the galaxy that the point `at` of `frame` lies inside,
/// as targets.
pub fn here(universe: &Universe, frame: Frame, at: DVec3) -> [Option<Target>; 3] {
    let (body, star, galaxy) = whereabouts(universe, frame, at);
    [
        body.map(Target::Body),
        star.map(Target::Star),
        galaxy.map(Target::Galaxy),
    ]
}
