//! Incoming creature batch ingestion, distributed lease arbitration and target mirroring.

use abstractions_net::ids::BodyId;
use abstractions_net::log::PlayerId;
use abstractions_net::wire::{Frame, Kind, ServerMsg, decode_fauna, records};
use bevy::prelude::*;

use super::{FaunaNet, LEASE, OwnerSlot, u8_to_stance};
use crate::fauna::{RemoteFauna, SyncTarget};
use crate::net::link::{FromServer, Inbox, Link};
use crate::space::Location;

/// Drops creature sync state on planet switches, player departures and lease expiry.
fn prune(
    net: &mut FaunaNet,
    remote: &mut RemoteFauna,
    body: Option<BodyId>,
    now: f64,
    msgs: &mut MessageReader<FromServer>,
) {
    if body != net.body {
        net.clear(remote);
        net.body = body;
    }
    for FromServer(msg) in msgs.read() {
        if let ServerMsg::Left(gone) = msg {
            net.owners.retain(|_, slot| slot.owner != *gone);
        }
    }
    net.owners.retain(|_, slot| now - slot.heard <= LEASE);
}

/// Applies all poses from `batch` using lowest-player-id arbitration and timestamp ordering.
fn apply_batch(
    net: &mut FaunaNet,
    (sender, you): (PlayerId, PlayerId),
    (sample_time, now): (f64, f64),
    bytes: &[u8],
) {
    let Some(decoded) = decode_fauna(bytes) else {
        return;
    };
    for pose in decoded.poses {
        let key = (pose.herd, pose.slot);
        if net.owned.contains(&key) && sender.0 > you.0 {
            continue;
        }
        if let Some(existing) = net.owners.get(&key) {
            if existing.owner != sender
                && sender.0 > existing.owner.0
                && now - existing.heard <= LEASE
            {
                continue;
            }
            if existing.owner == sender && sample_time < existing.target.time {
                continue;
            }
        }
        net.owned.remove(&key);
        let target = SyncTarget {
            species: pose.species,
            pos: pose.pos,
            vel: pose.vel,
            heading: pose.heading,
            stance: u8_to_stance(pose.stance),
            excite: pose.excite,
            time: sample_time,
        };
        net.owners.insert(
            key,
            OwnerSlot {
                owner: sender,
                heard: now,
                target,
            },
        );
    }
}

/// Mirrors active remote owner targets into [`RemoteFauna`] for fauna simulation systems.
fn sync_targets(net: &FaunaNet, remote: &mut RemoteFauna) {
    for (&key, slot) in &net.owners {
        remote.targets.insert(key, slot.target);
    }
    remote.targets.retain(|key, _| net.owners.contains_key(key));
}

/// Read-only resources inspected each frame when ingesting creature snapshots.
type RecvInputs<'w> = (
    Res<'w, Inbox>,
    Res<'w, Link>,
    Option<Res<'w, Location>>,
    Res<'w, Time<Real>>,
);

/// Processes incoming [`Kind::Fauna`] datagrams, player leaves and lease expiries before fauna step.
pub(super) fn ingest(
    (mut net, mut remote): (ResMut<FaunaNet>, ResMut<RemoteFauna>),
    (inbox, link, location, time): RecvInputs,
    mut msgs: MessageReader<FromServer>,
) {
    let now = time.elapsed_secs_f64();
    let body = location.as_deref().map(|loc| loc.body);
    prune(&mut net, &mut remote, body, now, &mut msgs);
    let (Some(you), Some(body)) = (link.you, body) else {
        remote.targets.clear();
        return;
    };
    let surface = Frame::Surface(body);
    for datagram in &inbox.0 {
        for record in records(datagram) {
            let head = record.head;
            if head.kind != Kind::Fauna || head.player == you || head.frame != surface {
                continue;
            }
            apply_batch(&mut net, (head.player, you), (head.time, now), record.body);
        }
    }
    sync_targets(&net, &mut remote);
}
