//! What the others hold, in the hands of their puppets: the tool of the recipe or the firearm tag
//! they sent, drawn and posed by the same systems as the local player.

use bevy::prelude::*;

use crate::avatar::strike::{Held, RemoteGun};
use crate::net::hunt::{Hunted, item};
use crate::net::state::Remotes;
use crate::world::Terrain;

/// Updates `puppet`'s held tool or firearm component to match `wanted` bytes.
fn apply_held(
    commands: &mut Commands,
    puppet: Entity,
    wanted: Option<&Vec<u8>>,
    (held, gun): (Option<&Held>, Option<&RemoteGun>),
    terrain: &Terrain,
) {
    let Some(bytes) = wanted else {
        if held.is_some() {
            commands.entity(puppet).remove::<Held>();
        }
        if gun.is_some() {
            commands.entity(puppet).remove::<RemoteGun>();
        }
        return;
    };
    if let Some(remote_gun) = item::unpack_gun(bytes) {
        if held.is_some() {
            commands.entity(puppet).remove::<Held>();
        }
        if gun != Some(&remote_gun) {
            commands.entity(puppet).try_insert(remote_gun);
        }
        return;
    }
    if gun.is_some() {
        commands.entity(puppet).remove::<RemoteGun>();
    }
    let same = held.is_some_and(|h| item::pack(&h.recipe).as_deref() == Some(bytes.as_slice()));
    if !same && let Some(tool) = item::unpack(bytes, terrain) {
        commands.entity(puppet).try_insert(tool);
    }
}

/// Puts the tool or firearm each other player holds in the hands of their puppet, and takes it
/// from the puppets whose player holds nothing.
pub fn dress(
    mut commands: Commands,
    hunted: Res<Hunted>,
    (remotes, terrain): (Res<Remotes>, Res<Terrain>),
    worn: Query<(Option<&Held>, Option<&RemoteGun>)>,
) {
    for (player, track) in &remotes.robots {
        let Some(puppet) = track.puppet else {
            continue;
        };
        let current = worn.get(puppet).unwrap_or((None, None));
        apply_held(
            &mut commands,
            puppet,
            hunted.holding.get(player),
            current,
            &terrain,
        );
    }
}
