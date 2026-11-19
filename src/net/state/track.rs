//! What arrives of the other players: per player and kind, the frame of its records, their
//! jitter buffer, when the last one came and the puppet that shows it.

use std::collections::HashMap;

use abstractions_net::jitter::{Buffer, Snap};
use abstractions_net::wire::{self, CompanionBody, Kind, Record, RobotBody, ShipBody, records};
use bevy::prelude::*;

use super::robot::{Stride, snap};
use crate::net::link::{Inbox, Link};
use crate::sim::PlayerId;

/// Seconds between two records of a moving thing, which the buffers plan their delay on.
const INTERVAL: f64 = super::rate::MOVING;

/// The records of one thing of one player.
#[derive(Debug, Clone)]
pub struct Track<E> {
    /// The frame of the records in the buffer.
    pub frame: wire::Frame,
    /// The records, a little in the past.
    pub buffer: Buffer<E>,
    /// Local time in seconds the last record came at.
    pub heard: f64,
    /// The puppet that shows the thing, once it is spawned.
    pub puppet: Option<Entity>,
    /// The name above a robot, once it is shown.
    pub tag: Option<Entity>,
    /// The distance a robot walked.
    pub stride: Stride,
    /// The body of the newest record.
    pub latest: E,
}

impl<E: Copy + Default> Track<E> {
    /// Returns a track with nothing in it yet, for records in `frame`.
    fn new(frame: wire::Frame) -> Self {
        Self {
            frame,
            buffer: Buffer::new(INTERVAL),
            heard: 0.0,
            puppet: None,
            tag: None,
            stride: Stride::default(),
            latest: E::default(),
        }
    }
}

/// The robots, ships and companions of the other players.
#[derive(Resource, Debug, Default)]
pub struct Remotes {
    /// The robots.
    pub robots: HashMap<PlayerId, Track<RobotBody>>,
    /// The ships.
    pub ships: HashMap<PlayerId, Track<ShipBody>>,
    /// The companions.
    pub companions: HashMap<PlayerId, Track<CompanionBody>>,
}

/// Keeps `snap` of the record of `head` that came at local time `now`. A record in another
/// frame than the ones before empties the buffer first.
fn keep<E: Copy + Default>(
    tracks: &mut HashMap<PlayerId, Track<E>>,
    head: &wire::Head,
    snap: Snap<E>,
    now: f64,
) {
    let track = tracks
        .entry(head.player)
        .or_insert_with(|| Track::new(head.frame));
    if track.frame != head.frame {
        track.buffer.reset();
        track.frame = head.frame;
    }
    if track.buffer.push(snap, now) {
        track.latest = snap.extra;
    }
    track.heard = now;
}

/// Returns the snapshot of a ship record sampled at universe time `time`.
fn hull(time: f64, body: ShipBody) -> Snap<ShipBody> {
    Snap {
        time,
        position: body.position,
        velocity: Some(body.velocity),
        rotation: body.rotation,
        extra: body,
    }
}

/// Returns the snapshot of a companion record sampled at universe time `time`. It carries no
/// velocity, so the buffer blends it in straight lines.
fn bot(time: f64, body: CompanionBody) -> Snap<CompanionBody> {
    Snap {
        time,
        position: body.position,
        velocity: None,
        rotation: body.rotation,
        extra: body,
    }
}

impl Remotes {
    /// Decodes `record`, which came at local time `now`, and keeps it. A body that does not
    /// decode is dropped.
    fn take(&mut self, record: &Record, now: f64) {
        let (head, time) = (&record.head, record.head.time);
        let (frame, bytes) = (&head.frame, record.body);
        match head.kind {
            Kind::Robot => {
                if let Some(body) = RobotBody::decode(frame, bytes) {
                    keep(&mut self.robots, head, snap(time, body), now);
                }
            }
            Kind::Ship => {
                if let Some(body) = ShipBody::decode(frame, bytes) {
                    keep(&mut self.ships, head, hull(time, body), now);
                }
            }
            Kind::Companion => {
                if let Some(body) = CompanionBody::decode(frame, bytes) {
                    keep(&mut self.companions, head, bot(time, body), now);
                }
            }
        }
    }

    /// Returns whether anything of another player is kept.
    pub fn any(&self) -> bool {
        !(self.robots.is_empty() && self.ships.is_empty() && self.companions.is_empty())
    }
}

/// Run condition: something of another player is kept.
pub fn kept(remotes: Res<Remotes>) -> bool {
    remotes.any()
}

/// Keeps every record of the batches of this frame, but those of this player.
pub fn ingest(inbox: Res<Inbox>, link: Res<Link>, mut remotes: ResMut<Remotes>) {
    let now = link.now();
    for batch in &inbox.0 {
        for record in records(batch) {
            if Some(record.head.player) != link.you {
                remotes.take(&record, now);
            }
        }
    }
}
