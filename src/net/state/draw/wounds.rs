//! The harm of the robots of the other players: each puppet carries the condition its vitals
//! say, which the walk reads to limp, as it does for the robot of this player.

use bevy::prelude::*;

use crate::condition::{Condition, Wounded};
use crate::net::state::Remote;
use crate::net::state::track::Remotes;

/// Gives the puppet of each robot the condition of its newest vitals, when it changed.
pub fn wounds(
    mut commands: Commands,
    remotes: Res<Remotes>,
    puppets: Query<&Wounded, With<Remote>>,
) {
    for (player, track) in &remotes.robots {
        let (Some(puppet), Some(vitals)) = (track.puppet, remotes.vitals.get(player)) else {
            continue;
        };
        let seen = Wounded(Condition::restored(vitals.harm.map(|harm| 1.0 - harm)));
        let known = puppets.get(puppet);
        let unchanged = known.map_or(seen.0.intact(), |old| *old == seen);
        if !unchanged {
            commands.entity(puppet).try_insert(seen);
        }
    }
}
