//! The room: the server's logic with no network in it. Messages and datagrams go in, messages
//! and datagrams come out. The server and a game open to LAN run the same room.
//!
//! The room never simulates and never decodes a state body. It owns the world (seed, clock,
//! log, atlas, player records, hunt), lets players in, keeps the last few states of each player
//! and kind, and copies them to the others every tick: all of them around the same star, the
//! newest once a second elsewhere.

mod conn;
mod host;
mod hunt;
mod join;
mod map;
mod play;
mod relay;
mod seat;

#[cfg(test)]
mod explored_tests;
#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod hunt_kit;
#[cfg(test)]
mod hunt_tests;
#[cfg(test)]
mod join_tests;
#[cfg(test)]
mod keep_tests;
#[cfg(test)]
mod kit;
#[cfg(test)]
mod map_tests;
#[cfg(test)]
mod play_tests;
#[cfg(test)]
mod relay_tests;
#[cfg(test)]
mod scale_tests;

use std::collections::HashMap;

pub use map::{BODIES_MAX, RUNS_MAX};

use conn::Conn;
use hunt::Hunting;
use map::Ground;
use seat::{Seat, Sent};

use crate::host::{ConnId, DATAGRAM_BUDGET};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::store::World;
use crate::wire::{Kind, PlayerInfo};

/// What the owner of a server chose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomConfig {
    /// Name of the server, shown in the server list.
    pub name: String,
    /// Password a player must give to join, or nothing for an open server.
    pub password: Option<String>,
    /// Most players in the world at once.
    pub cap: u16,
    /// Version of the server program, such as `0.9.0`.
    pub version: String,
}

/// The server's logic. It implements [`crate::host::Host`].
#[derive(Debug)]
pub struct Room {
    /// What the owner chose.
    config: RoomConfig,
    /// The world as loaded, with the log and the player records kept up to date.
    world: World,
    /// Network time when the room started, which matches `world.clock`.
    start: f64,
    /// Every open connection, joined or not.
    conns: HashMap<ConnId, Conn>,
    /// One seat per player the room takes, empty when free.
    seats: Vec<Option<Seat>>,
    /// What each seat last got of each other seat and kind: `sent[to][from][kind]`.
    sent: Vec<Vec<[Sent; Kind::COUNT]>>,
    /// The id each identity key had this session, so a player who comes back keeps it.
    ids: HashMap<[u8; 32], PlayerId>,
    /// The next id never given yet. It starts at 1: 0 is the single-player id.
    next_id: u32,
    /// The datagram batch being filled, reused every tick.
    batch: Vec<u8>,
    /// Number of the last state record kept, so newer records compare greater.
    seq: u64,
    /// Value of `seq` at the end of the last relay: records above it came since that tick.
    relayed: u64,
    /// The explored ground of each body.
    explored: HashMap<BodyId, Ground>,
    /// The hurt animals, the carcasses, the stuck spears and the held items.
    hunting: Hunting,
    /// True when something that is saved changed since [`Room::saved`] was last called.
    dirty: bool,
    /// Network time of the last sweep of the hunt.
    swept: f64,
}

impl Room {
    /// Returns a room for `world` with nobody in it. `now` is the network time, so the
    /// universe clock reads `world.clock` at that moment.
    pub fn new(config: RoomConfig, mut world: World, now: f64) -> Self {
        let cap = usize::from(config.cap);
        let explored = map::grounds(std::mem::take(&mut world.explored));
        let hunting = Hunting::new(std::mem::take(&mut world.hunt));
        Self {
            config,
            world,
            start: now,
            conns: HashMap::new(),
            seats: (0..cap).map(|_| None).collect(),
            sent: vec![vec![[Sent::NONE; Kind::COUNT]; cap]; cap],
            ids: HashMap::new(),
            next_id: 1,
            batch: Vec::with_capacity(2 * DATAGRAM_BUDGET),
            seq: 0,
            relayed: 0,
            explored,
            hunting,
            dirty: false,
            swept: f64::NEG_INFINITY,
        }
    }

    /// Returns the world to save: the universe clock advanced to `now`, every record kept, the
    /// explored ground and the hunt. The runs of each body come from the copy kept when its
    /// ground changed, so nothing is rebuilt here.
    pub fn snapshot(&self, now: f64) -> World {
        World {
            clock: self.clock(now),
            explored: self.explored_list(),
            hunt: self.hunting.world(Vec::new()),
            ..self.world.clone()
        }
    }

    /// Returns true when something that is saved changed since [`Room::saved`] was last called:
    /// a change of the log, a player's record, explored ground, a carcass or a spear. A server
    /// that finds it false has nothing new to write.
    pub fn changed(&self) -> bool {
        self.dirty
    }

    /// Says the world was handed to be saved, so [`Room::changed`] is false until the next change.
    pub fn saved(&mut self) {
        self.dirty = false;
    }

    /// Returns the players in the world now, by id.
    pub fn players(&self) -> Vec<PlayerInfo> {
        let mut players: Vec<PlayerInfo> = self
            .seats
            .iter()
            .flatten()
            .map(|seat| seat.info.clone())
            .collect();
        players.sort_by_key(|player| player.id.0);
        players
    }

    /// Returns the universe time at network time `now`. It never pauses.
    fn clock(&self, now: f64) -> f64 {
        self.world.clock + (now - self.start)
    }
}
