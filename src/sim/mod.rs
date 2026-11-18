//! The simulation contract that keeps the game ready for more than one player.
//!
//! The world is the seed plus a log of what players changed. Players act through intents, which
//! the rules read instead of the keyboard, so a remote player is the same message from another
//! source. Every rule takes a player by [`PlayerId`]; only the camera, the interface and the
//! input code may assume there is one player.

use serde::{Deserialize, Serialize};

/// A player of the world. The local player of a single-player game is [`PlayerId::LOCAL`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

impl PlayerId {
    /// The player at this machine.
    pub const LOCAL: Self = Self(0);
}

/// Simulation steps per second: the rules that must agree between players run at this rate.
pub const STEPS_PER_SECOND: f64 = 30.0;

/// One change a player made to a world that the seed alone cannot give back.
///
/// Later versions add kills, harvested deposits, dropped gear and dug ground; each kind is
/// added at the end, so older logs still read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Change {
    /// A body was visited for the first time.
    Visited(crate::universe::BodyId),
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bevy::prelude::Resource)]
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
