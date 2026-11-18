//! Rooms among the built floors: which floor under a point is the floor of a room, with a roof over it,
//! and not a platform, a plaza or a boardwalk under the open sky.

use bevy::math::DVec3;

use super::{Surroundings, Survey};
use crate::world::structures::pads::FloorKind;

impl Survey {
    /// Returns what the floor of the room is made of when feet at the planet position `pos` stand in one:
    /// inside the walls of a village house or on the floor of a small structure. Streets, plazas and the
    /// platforms around the houses are no rooms, whatever they are made of.
    pub fn room_at(&self, pos: DVec3) -> Option<FloorKind> {
        self.pads.iter().find_map(|pad| pad.room(pos))
    }
}

impl Surroundings {
    /// Returns what the floor of the room around feet at the planet position `pos` is made of, or nothing
    /// outside every room. Complete within [`super::AROUND`] meters of the player.
    pub fn room_at(&self, pos: DVec3) -> Option<FloorKind> {
        self.survey.room_at(pos)
    }
}
