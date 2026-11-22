//! Capturing where and when the player is, for the save.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::model::{self, GAME_VERSION, Place, SAVE_VERSION, WorldSave};
use crate::computer::Atlas;
use crate::map::explored::Explored;
use crate::sim::Changes;
use crate::space::{Anchor, Clock, Frame, Location, Travel};
use crate::universe::{BodyId, EARTH_RADIUS, Universe};
use crate::world::{Link, Planet, Terrain};

/// Returns the spot of render position `at` on the planet the anchor is fixed to.
///
/// Returns nothing off a planet and on the legacy flat world, which has no directions.
pub(super) fn place(anchor: &Anchor, radius: f64, at: Vec3) -> Option<Place> {
    let Frame::Surface(body) = anchor.frame else {
        return None;
    };
    let pos = anchor.to_frame(at);
    let dir = pos.try_normalize().filter(|_| !anchor.flat)?;
    Some(Place {
        body,
        dir: dir.to_array(),
        altitude: pos.length() - radius,
    })
}

/// Returns the spot of render position `at` against the planet `body` of sea level `radius`, for an
/// anchor in the frame of its star: `link` goes from the frame of the planet to that of the anchor.
/// The spot turns with the planet and its height is any number of metres, as in orbit.
///
/// Returns nothing on a surface, where [`place`] reads the spot, and between the stars.
pub(super) fn beside(
    anchor: &Anchor,
    link: &Link,
    (body, radius): (BodyId, f64),
    at: Vec3,
) -> Option<Place> {
    let Frame::System(_) = anchor.frame else {
        return None;
    };
    let pos = link.to_planet(anchor.to_frame(at));
    let dir = pos.try_normalize()?;
    Some(Place {
        body,
        dir: dir.to_array(),
        altitude: pos.length() - radius,
    })
}

/// Writes where the body at render position `at` stands: a place on a globe, or the flat
/// position on the legacy flat world. Off a planet the last place is kept.
pub(super) fn stand(save: &mut WorldSave, anchor: Option<&Anchor>, radius: f64, at: Vec3) {
    match anchor {
        Some(anchor) if !anchor.flat => {
            if let Some(place) = place(anchor, radius, at) {
                save.place = Some(place);
                save.position = None;
            }
        }
        _ => save.position = Some(at.to_array()),
    }
}

/// The parts of the world a save records besides the player pose.
#[derive(SystemParam)]
pub(super) struct Site<'w> {
    clock: Res<'w, Clock>,
    anchor: Option<Res<'w, Anchor>>,
    location: Option<Res<'w, Location>>,
    terrain: Option<Res<'w, Terrain>>,
    atlas: Option<Res<'w, Atlas>>,
    changes: Option<Res<'w, Changes>>,
    /// The ground uncovered on each body.
    explored: Option<Res<'w, Explored>>,
    /// The planet the world streams: the nearest one while in space.
    planet: Option<Res<'w, Planet>>,
    /// Where the planets are.
    universe: Option<Res<'w, Universe>>,
}

impl Site<'_> {
    /// Writes the format version, the universe time, the atlas, the log and the explored ground
    /// into `save`.
    pub(super) fn record(&self, save: &mut WorldSave) {
        save.version = SAVE_VERSION;
        save.game_version = GAME_VERSION.to_string();
        save.clock_seconds = self.clock.seconds;
        if let Some(atlas) = self.atlas.as_deref() {
            save.atlas = Some(atlas.clone());
        }
        if let Some(changes) = self.changes.as_deref() {
            save.changes = changes.clone();
        }
        if let Some(explored) = self.explored.as_deref() {
            let bodies = explored.bodies().map(|(body, leaves)| model::Explored {
                body,
                runs: leaves.runs(),
            });
            save.explored = bodies.collect();
        }
    }

    /// Returns true while the player walks or swims, and the view angles are the player's own.
    pub(super) fn walking(&self) -> bool {
        let travel = self.location.as_deref().map(|location| location.travel);
        travel != Some(Travel::InShip)
    }

    /// Writes where the player body at render position `at` stands. A seated body sits in
    /// the ship, so this is the place of the ship: over a planet, and in the space of a system
    /// its spot against the nearest planet, however high. Only between the stars is the last
    /// place kept; the record of the ship then says where the player is.
    pub(super) fn stand(&self, save: &mut WorldSave, at: Vec3) {
        let surface = match self.anchor.as_deref().map(|a| a.frame) {
            Some(Frame::Surface(body)) => self
                .planet
                .as_deref()
                .filter(|p| p.body == body)
                .map(|p| p.globe.radius())
                .or_else(|| self.universe.as_deref().map(|u| u.planet(body).radius)),
            _ => None,
        };
        let terrain = self.terrain.as_deref().map(|t| t.globe().radius());
        let radius = surface.or(terrain).unwrap_or(EARTH_RADIUS);
        stand(save, self.anchor.as_deref(), radius, at);
        if let Some(place) = self.orbiting(at) {
            save.place = Some(place);
        }
    }

    /// Returns the spot of render position `at` against the streamed planet while the anchor is
    /// in the frame of its star.
    fn orbiting(&self, at: Vec3) -> Option<Place> {
        let (anchor, planet) = (self.anchor.as_deref()?, self.planet.as_deref()?);
        let universe = self.universe.as_deref()?;
        let link = Link::between(anchor.frame, planet, universe, self.clock.seconds)?;
        let body = (planet.body, planet.globe.radius());
        beside(anchor, &link, body, at)
    }
}
