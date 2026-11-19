//! Where the player is on a planet, for the map: the planet and the point of it in metres from
//! its centre, in the frame that turns with it. Seated, that is the ship.

use bevy::ecs::system::SystemParam;
use bevy::math::DVec3;
use bevy::prelude::*;

use crate::player::Player;
use crate::ship::Ship;
use crate::space::{Anchor, Frame, Location, Travel};
use crate::universe::BodyId;

/// What the place of the player is read from.
#[derive(SystemParam)]
pub struct Me<'w, 's> {
    /// The render anchor.
    anchor: Option<Res<'w, Anchor>>,
    /// Whether the player walks or sits in the ship.
    location: Option<Res<'w, Location>>,
    /// The ship.
    ships: Query<'w, 's, &'static Ship>,
    /// The body of the player.
    bodies: Query<'w, 's, &'static GlobalTransform, With<Player>>,
}

impl Me<'_, '_> {
    /// Returns the planet the player is on or flies over and the point of it where the player
    /// is, or nothing in space and on the legacy flat world.
    pub fn spot(&self) -> Option<(BodyId, DVec3)> {
        let anchor = self.anchor.as_deref().filter(|anchor| !anchor.flat)?;
        let Frame::Surface(body) = anchor.frame else {
            return None;
        };
        let seated = self.location.as_deref().map(|at| at.travel) == Some(Travel::InShip);
        let ship = self.ships.iter().find(|ship| ship.frame == anchor.frame);
        if let Some(ship) = ship.filter(|_| seated) {
            return Some((body, ship.pos));
        }
        let at = self.bodies.iter().next()?.translation();
        Some((body, anchor.to_frame(at)))
    }
}
