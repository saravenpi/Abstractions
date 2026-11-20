//! What the others hold, in the hands of their puppets: the tool of the recipe they sent, drawn
//! and posed by the same systems as the tool of this player.

use bevy::prelude::*;

use crate::avatar::strike::Held;
use crate::net::hunt::{Hunted, item};
use crate::net::state::Remotes;
use crate::world::Terrain;

/// Puts the tool each other player holds in the hands of their puppet, and takes it from the
/// puppets whose player holds nothing.
pub fn dress(
    mut commands: Commands,
    hunted: Res<Hunted>,
    (remotes, terrain): (Res<Remotes>, Res<Terrain>),
    worn: Query<&Held>,
) {
    for (player, track) in &remotes.robots {
        let Some(puppet) = track.puppet else {
            continue;
        };
        let wanted = hunted.holding.get(player);
        let same = |held: &Held| item::pack(&held.recipe).as_ref() == wanted;
        match (wanted, worn.get(puppet)) {
            (None, Ok(_)) => {
                commands.entity(puppet).remove::<Held>();
            }
            (Some(_), Ok(held)) if same(held) => {}
            (Some(bytes), _) => {
                if let Some(held) = item::unpack(bytes, &terrain) {
                    commands.entity(puppet).try_insert(held);
                }
            }
            (None, Err(_)) => {}
        }
    }
}
