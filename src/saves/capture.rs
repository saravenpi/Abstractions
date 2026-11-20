//! Capturing where and when the player is, for the save.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::model::{self, Place, SAVE_VERSION, WorldSave};
use crate::computer::Atlas;
use crate::map::explored::Explored;
use crate::sim::Changes;
use crate::space::{Anchor, Clock, Frame, Location, Travel};
use crate::universe::EARTH_RADIUS;
use crate::world::Terrain;

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
}

impl Site<'_> {
    /// Writes the format version, the universe time, the atlas, the log and the explored ground
    /// into `save`.
    pub(super) fn record(&self, save: &mut WorldSave) {
        save.version = SAVE_VERSION;
        save.clock_seconds = self.clock.seconds;
        if let Some(atlas) = self.atlas.as_deref() {
            save.atlas = Some(atlas.clone());
        }
        if let Some(changes) = self.changes.as_deref() {
            save.changes = changes.clone();
        }
        if let Some(explored) = self.explored.as_deref() {
            let bodies = explored.bodies().map(|(body, words)| model::Explored {
                body,
                words: words.to_vec(),
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
    /// the ship, so near a planet this is the place of the ship, and in space the last place
    /// on a planet is kept.
    pub(super) fn stand(&self, save: &mut WorldSave, at: Vec3) {
        let terrain = self.terrain.as_deref();
        let radius = terrain.map_or(EARTH_RADIUS, |terrain| terrain.globe().radius());
        stand(save, self.anchor.as_deref(), radius, at);
    }
}
