//! The world file of a server: name, seed, clock, log, atlas and player records, written
//! safely.
//!
//! The file is TOML at `world.toml` in the server's data folder. [`save`] writes it whole
//! through a temporary file, so a crash leaves either the old file or the new one, and keeps
//! the last three copies. The explored ground goes beside it in `explored3.bin`, written the
//! same safe way. [`import`] reads a single-player world of the game.

mod explored;
mod file;
mod import;

#[cfg(test)]
mod explored_tests;
#[cfg(test)]
mod file_tests;
#[cfg(test)]
mod import_tests;

pub use explored::EXPLORED_FILE;
pub use file::{FILE, load_or_create, save};
pub use import::import;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;
use crate::leaves::Run;
use crate::log::Changes;
use crate::wire::PlayerRecord;

/// What the server keeps of one player between sessions.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Member {
    /// The name the player used last.
    pub name: String,
    /// Where the player was last, as the game wrote it.
    pub place: String,
    /// The player's ship, as the game wrote it.
    pub ship: String,
    /// The player's saved places, as the game wrote them.
    pub marks: String,
}

impl Member {
    /// Returns the record the game reads back on join.
    pub fn record(&self) -> PlayerRecord {
        PlayerRecord {
            place: self.place.clone(),
            ship: self.ship.clone(),
            marks: self.marks.clone(),
        }
    }
}

/// Everything a server's world file stores. Missing keys take their defaults, so older files
/// still load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct World {
    /// Name of the world, shown in the server list.
    pub name: String,
    /// Seed the whole universe is generated from.
    pub seed: u64,
    /// Universe time in seconds when the file was written.
    pub clock: f64,
    /// Every change the players made, oldest first.
    pub changes: Changes,
    /// The shared atlas as TOML text. The server stores it and never reads it.
    pub atlas: String,
    /// Every player who ever kept a record here, by identity key in hexadecimal.
    pub players: BTreeMap<String, Member>,
    /// The ground explored on each body, as runs of leaves.
    /// It lives in [`EXPLORED_FILE`], not in the world file.
    #[serde(skip)]
    pub explored: Vec<(BodyId, Vec<Run>)>,
}

impl Default for World {
    /// Returns an empty world named `World` with seed 0.
    fn default() -> Self {
        Self {
            name: "World".to_string(),
            seed: 0,
            clock: 0.0,
            changes: Changes::default(),
            atlas: String::new(),
            players: BTreeMap::new(),
            explored: Vec::new(),
        }
    }
}
