//! The log of what players changed in a world: the part of a world the seed cannot give back.
//!
//! The game saves it with the world and the server sends it to every player who joins.

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;

/// A player of the world. The local player of a single-player game is [`PlayerId::LOCAL`].
///
/// Every player body carries its id, and the rules find a player's body by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::component::Component))]
pub struct PlayerId(pub u32);

impl PlayerId {
    /// The player at this machine.
    pub const LOCAL: Self = Self(0);
}

/// One change a player made to a world that the seed alone cannot give back.
///
/// Later versions add kills, harvested deposits, dropped gear and dug ground; each kind is
/// added at the end, so older logs still read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Change {
    /// A body was visited for the first time.
    Visited(BodyId),
}

/// One entry of the log: who changed what, and when in universe seconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    /// The player who made the change.
    pub player: PlayerId,
    /// Universe time of the change in seconds.
    pub time: f64,
    /// What changed.
    pub change: Change,
}

/// The append-only log of changes, saved with the world.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ecs", derive(bevy_ecs::resource::Resource))]
pub struct Changes {
    /// Every change, oldest first.
    pub entries: Vec<Entry>,
}

impl Changes {
    /// Adds a change at the end of the log.
    pub fn push(&mut self, player: PlayerId, time: f64, change: Change) {
        self.entries.push(Entry {
            player,
            time,
            change,
        });
    }
}
