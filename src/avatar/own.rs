//! Finding the body of the player at this machine among every body drawn.
//!
//! Every body is posed from its own [`super::AvatarPose`]. Only the systems that read this
//! machine's controller, gear and camera need to know which body is the local one.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Avatar;
use crate::player::{Player, local};
use crate::sim::PlayerId;

/// The players and the bodies, to pair the local player with its body.
#[derive(SystemParam)]
pub struct Own<'w, 's> {
    /// Every player entity with the id of its player.
    players: Query<'w, 's, (Entity, &'static PlayerId), With<Player>>,
    /// Every body with the entity it hangs from.
    bodies: Query<'w, 's, (Entity, &'static ChildOf), With<Avatar>>,
}

impl Own<'_, '_> {
    /// Returns the entity of the player at this machine and the root of its body, once it has one.
    pub fn body(&self) -> Option<(Entity, Entity)> {
        let player = local(self.players.iter().map(|(entity, id)| (*id, entity)))?;
        let mut bodies = self.bodies.iter();
        let found = bodies.find(|(_, parent)| parent.parent() == player)?;
        Some((player, found.0))
    }
}
