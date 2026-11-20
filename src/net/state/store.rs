//! What is kept of the other players, one map per kind of record, all taken and swept the same
//! way: a map of [`Track`]s for the things that move and are drawn from a jitter buffer, a map
//! of [`Held`](super::held::Held) bodies for the things that only say how a thing is.

use std::collections::HashMap;

use abstractions_net::jitter::Snap;
use abstractions_net::wire::{Body, CompanionBody, Record, ShipBody};
use bevy::prelude::*;

use super::draw::drop_puppet;
use super::rebase::rebase;
use super::track::{Arrival, Track};
use crate::sim::PlayerId;

/// A body that moves: it can be sampled between two records.
pub trait Tracked: Body {
    /// Returns the snapshot of this body sampled at universe time `time`.
    fn snap(self, time: f64) -> Snap<Self>;
}

/// The records of one kind, per player.
pub trait Store {
    /// Keeps the body of `record` when it is of this kind of record and decodes. Anything else
    /// is left alone.
    fn take(&mut self, record: &Record, arrival: &mut Arrival);

    /// Keeps the things for which `keep` holds, given the player and when the thing was last
    /// heard, and despawns the puppets of the others.
    fn sweep(&mut self, commands: &mut Commands, keep: &dyn Fn(PlayerId, f64) -> bool);
}

impl<B: Tracked> Store for HashMap<PlayerId, Track<B>> {
    /// Keeps the snapshot of the record. A record in another frame than the ones before moves
    /// the buffer into its frame first.
    fn take(&mut self, record: &Record, arrival: &mut Arrival) {
        let head = &record.head;
        if head.kind != B::KIND {
            return;
        }
        let Some(body) = B::decode(&head.frame, record.body) else {
            return;
        };
        let snap = body.snap(head.time);
        let track = self
            .entry(head.player)
            .or_insert_with(|| Track::new(head.frame));
        if track.frame != head.frame {
            rebase(&mut track.buffer, (track.frame, head.frame), arrival.bodies);
            track.frame = head.frame;
        }
        if track.buffer.push(snap, arrival.universe) {
            track.latest = snap.extra;
        }
        track.heard = arrival.local;
    }

    /// Keeps the tracks for which `keep` holds and despawns the puppets of the others.
    fn sweep(&mut self, commands: &mut Commands, keep: &dyn Fn(PlayerId, f64) -> bool) {
        self.retain(|player, track| {
            let kept = keep(*player, track.heard);
            if !kept {
                drop_puppet(commands, track);
            }
            kept
        });
    }
}

impl Tracked for ShipBody {
    /// Returns the snapshot of a ship record: it has a velocity, so the buffer carries it on.
    fn snap(self, time: f64) -> Snap<Self> {
        Snap {
            time,
            position: self.position,
            velocity: Some(self.velocity),
            rotation: self.rotation,
            extra: self,
        }
    }
}

impl Tracked for CompanionBody {
    /// Returns the snapshot of a companion record. It carries no velocity, so the buffer blends
    /// it in straight lines.
    fn snap(self, time: f64) -> Snap<Self> {
        Snap {
            time,
            position: self.position,
            velocity: None,
            rotation: self.rotation,
            extra: self,
        }
    }
}
