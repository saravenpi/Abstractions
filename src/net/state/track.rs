//! What arrives of the other players: per player and kind, the frame of its records, their
//! jitter buffer, when the last one came and the puppet that shows it.

use std::collections::HashMap;

use abstractions_net::jitter::Buffer;
use abstractions_net::wire::{self, CompanionBody, Kind, MOVING, Record, RobotBody, ShipBody};
use abstractions_net::wire::{VitalsBody, records};
use bevy::prelude::*;

use super::bodies::Bodies;
use super::held::Held;
use super::robot::Stride;
use super::store::Store;
use crate::net::link::{Inbox, Link};
use crate::ship::Fleet;
use crate::sim::PlayerId;
use crate::space::Clock;
use crate::universe::Universe;

/// Seconds between two records of a moving thing, which the buffers plan their delay on.
const INTERVAL: f64 = MOVING;

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
    pub(super) fn new(frame: wire::Frame) -> Self {
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
    pub vitals: HashMap<PlayerId, Held<VitalsBody>>,
    /// Active ship pilots mapped by ship owner id to current pilot player id.
    pub pilots: HashMap<PlayerId, PlayerId>,
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

impl Remotes {
    /// Returns the current pilot of `ship` when another or local player sits at its controls.
    pub fn pilot_of(&self, ship: PlayerId) -> Option<PlayerId> {
        self.pilots.get(&ship).copied()
    }

    /// Returns the owner id of the ship currently piloted by `pilot`, if any.
    pub fn ship_of(&self, pilot: PlayerId) -> Option<PlayerId> {
        self.pilots
            .iter()
            .find_map(|(ship, active)| (*active == pilot).then_some(*ship))
    }

    /// Returns the map of every kind of record, to take and sweep them alike.
    pub(super) fn stores(&mut self) -> [&mut dyn Store; Kind::COUNT] {
        [
            &mut self.robots,
            &mut self.ships,
            &mut self.companions,
            &mut self.vitals,
        ]
    }

    /// Keeps `record`, which came at `now`, in the map of its kind. A body that does not decode
    /// is dropped.
    fn take(&mut self, record: &Record, now: &mut Arrival) {
        for store in self.stores() {
            store.take(record, now);
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

/// Keeps every record of the batches of this frame, skipping our own active records and this
/// player's robot/companion records.
pub fn ingest(
    (inbox, link, fleet): (Res<Inbox>, Res<Link>, Option<Res<Fleet>>),
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
    let active_ship = fleet.as_deref().and_then(|f| f.active);
    for batch in &inbox.0 {
        for record in records(batch) {
            let skip = if record.head.kind == Kind::Ship {
                Some(record.head.player) == active_ship
            } else {
                Some(record.head.player) == link.you
            };
            if !skip {
                remotes.take(&record, &mut now);
            }
        }
    }
}
