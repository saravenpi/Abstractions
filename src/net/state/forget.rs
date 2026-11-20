//! Letting go of other players: those who left, those not heard from for [`STALE`] seconds,
//! and everyone once the session is over.

use abstractions_net::wire::ServerMsg;
use bevy::prelude::*;

use super::track::Remotes;
use crate::net::link::{FromServer, Link};
use crate::sim::PlayerId;

/// Seconds without a record after which the things of a player are forgotten.
pub const STALE: f64 = 10.0;

impl Remotes {
    /// Keeps the things of the players for which `keep` holds, given the player and when the
    /// thing was last heard, and despawns the puppets of the others.
    fn retain(&mut self, commands: &mut Commands, keep: impl Fn(PlayerId, f64) -> bool) {
        for store in self.stores() {
            store.sweep(commands, &keep);
        }
    }
}

/// Forgets the players who left the session and those not heard from for [`STALE`] seconds.
pub fn forget(
    mut commands: Commands,
    mut remotes: ResMut<Remotes>,
    link: Res<Link>,
    mut from: MessageReader<FromServer>,
) {
    for FromServer(msg) in from.read() {
        if let ServerMsg::Left(gone) = msg {
            remotes.retain(&mut commands, |player, _| player != *gone);
        }
    }
    let now = link.now();
    remotes.retain(&mut commands, |_, heard| now - heard < STALE);
}

/// Forgets every other player once the session is over.
pub fn clear(mut commands: Commands, mut remotes: ResMut<Remotes>) {
    remotes.retain(&mut commands, |_, _| false);
}
