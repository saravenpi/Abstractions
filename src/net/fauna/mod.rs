//! Multiplayer creature synchronization: distributed per-creature ownership, incoming snapshot
//! ingestion and outgoing 10 Hz batched pose broadcasting.

mod recv;
mod send;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use abstractions_net::ids::BodyId;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::FaunaPose;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

use super::link::{Link, Receive};
use crate::fauna::senses::Stance;
use crate::fauna::{Posed, RemoteFauna, SyncTarget};

/// Seconds between outgoing creature pose batches per authoritative client.
pub(super) const FAUNA_EVERY: f64 = 0.10;

/// Seconds without updates before a remote owner's creature lease expires.
pub(super) const LEASE: f64 = 0.75;

/// Active remote authority lease and target state for one creature.
#[derive(Debug, Clone, Copy)]
pub(super) struct OwnerSlot {
    /// Player currently authoring snapshots for this creature.
    pub(super) owner: PlayerId,
    /// Local [`Time<Real>`] timestamp when the last snapshot from `owner` arrived.
    pub(super) heard: f64,
    /// Latest decoded target pose and mood state.
    pub(super) target: SyncTarget,
}

/// Client-side state for distributed creature authority and reusable send buffers.
#[derive(Resource, Debug, Default)]
pub struct FaunaNet {
    /// Planet whose creatures this sync state applies to.
    pub(super) body: Option<BodyId>,
    /// Creatures currently owned by remote players, keyed by `(herd_or_carcass, slot)`.
    pub(super) owners: HashMap<(u64, u8), OwnerSlot>,
    /// Creatures currently drawn and authored by this client.
    pub(super) owned: HashSet<(u64, u8)>,
    /// Local [`Time<Real>`] timestamp when the next outgoing batch may be sent.
    pub(super) next_send: f64,
    /// Reused datagram batch buffer.
    pub(super) batch: Vec<u8>,
    /// Reused encoded body buffer.
    pub(super) buf: Vec<u8>,
    /// Reused list of outgoing poses collected this tick.
    pub(super) poses: Vec<FaunaPose>,
}

impl FaunaNet {
    /// Clears tracked remote owners, local claims and pending target state.
    pub(super) fn clear(&mut self, remote: &mut RemoteFauna) {
        self.body = None;
        self.owners.clear();
        self.owned.clear();
        self.next_send = 0.0;
        remote.targets.clear();
    }
}

/// Maps a local fauna [`Stance`] to its wire `u8` representation.
pub(super) fn stance_to_u8(stance: Stance) -> u8 {
    match stance {
        Stance::Calm => 0,
        Stance::Wary => 1,
        Stance::Flee => 2,
        Stance::Threaten => 3,
        Stance::Attack => 4,
    }
}

/// Maps a wire `u8` stance code back to a local [`Stance`].
pub(super) fn u8_to_stance(code: u8) -> Stance {
    match code {
        1 => Stance::Wary,
        2 => Stance::Flee,
        3 => Stance::Threaten,
        4 => Stance::Attack,
        _ => Stance::Calm,
    }
}

/// Clears creature sync state once the network link is removed.
fn clear_offline(mut net: ResMut<FaunaNet>, mut remote: ResMut<RemoteFauna>) {
    if net.body.is_some()
        || !net.owners.is_empty()
        || !net.owned.is_empty()
        || !remote.targets.is_empty()
    {
        net.clear(&mut remote);
    }
}

/// Registers incoming and outgoing multiplayer creature synchronization systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<RemoteFauna>()
        .init_resource::<FaunaNet>()
        .add_systems(
            Update,
            (
                recv::ingest
                    .after(Receive)
                    .before(Posed)
                    .run_if(resource_exists::<Link>),
                clear_offline.run_if(not(resource_exists::<Link>)),
            ),
        )
        .add_systems(
            PostUpdate,
            send::send
                .after(TransformSystems::Propagate)
                .run_if(resource_exists::<Link>),
        );
}
