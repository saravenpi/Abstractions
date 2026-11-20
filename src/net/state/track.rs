//! What arrives of the other players: per player and kind, the frame of its records, their
//! jitter buffer, when the last one came and the puppet that shows it.

use std::collections::HashMap;

use abstractions_net::jitter::{Buffer, Snap};
use abstractions_net::wire::records;
use abstractions_net::wire::{
    self, Body, CompanionBody, Kind, Record, RobotBody, ShipBody, VitalsBody,
};
use bevy::prelude::*;

use super::bodies::Bodies;
use super::rebase::rebase;
use super::robot::{Stride, snap};
use crate::net::link::{Inbox, Link};
use crate::sim::PlayerId;
use crate::space::Clock;
use crate::universe::Universe;

/// Seconds between two records of a moving thing, which the buffers plan their delay on.
const INTERVAL: f64 = abstractions_net::wire::MOVING;

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
    /// How hurt each robot is and how it glows, as the newest record says.
    pub vitals: HashMap<PlayerId, VitalsBody>,
}

/// When a record came, on the universe clock the buffers sample on and on the local clock the
/// players are let go by, and the bodies its frame is converted against.
pub struct Arrival<'a> {
    /// Universe time in seconds, the clock of the jitter buffers.
    pub universe: f64,
    /// Local time in seconds.
    pub local: f64,
    /// The bodies frames are converted against.
    pub bodies: &'a mut Bodies,
}

/// Keeps `snap` of the record of `head` that came at `arrival`. A record in another frame than
/// the ones before moves the buffer into its frame first.
fn keep<E: Copy + Default>(
    tracks: &mut HashMap<PlayerId, Track<E>>,
    head: &wire::Head,
    snap: Snap<E>,
    arrival: &mut Arrival,
) {
    let track = tracks
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
    /// Decodes `record`, which came at `now`, and keeps it. A body that does not decode is
    /// dropped.
    fn take(&mut self, record: &Record, now: &mut Arrival) {
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
            Kind::Vitals => {
                if let Some(body) = VitalsBody::decode(frame, bytes) {
                    self.vitals.insert(head.player, body);
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

/// Keeps every record of the batches of this frame, but those of this player. Each one came
/// at the universe time of the frame.
pub fn ingest(
    (inbox, link): (Res<Inbox>, Res<Link>),
    mut remotes: ResMut<Remotes>,
    (mut bodies, clock, universe): (ResMut<Bodies>, Res<Clock>, Option<Res<Universe>>),
) {
    if let Some(universe) = universe.as_deref() {
        bodies.sync(universe);
    }
    let mut now = Arrival {
        universe: clock.seconds,
        local: link.now(),
        bodies: &mut bodies,
    };
    for batch in &inbox.0 {
        for record in records(batch) {
            if Some(record.head.player) != link.you {
                remotes.take(&record, &mut now);
            }
        }
    }
}
