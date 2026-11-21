//! The world file of a server: name, seed, clock, log, atlas and player records, written
//! safely.
//!
//! The file is TOML at `world.toml` in the server's data folder. [`save`] writes it whole
//! through a temporary file, so a crash leaves either the old file or the new one, and keeps
//! the last three copies. The explored ground goes beside it in `explored3.bin`, the hunt in
//! `hunt3.bin` and the things on the ground in `ground3.bin`, written the same safe way.
//! [`import`] reads a single-player world of the game. [`place`] checks the place kept of each
//! player.

mod explored;
mod file;
mod hunt;
mod import;
pub mod place;

#[cfg(test)]
mod explored_tests;
#[cfg(test)]
mod file_tests;
#[cfg(test)]
mod ground_tests;
#[cfg(test)]
mod home_tests;
#[cfg(test)]
mod import_tests;
#[cfg(test)]
mod place_tests;

pub use explored::EXPLORED_FILE;
pub use file::{FILE, load_or_create, save};
pub use hunt::{GROUND_FILE, HUNT_FILE};
pub use import::import;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;
use crate::leaves::Run;
use crate::log::Changes;
use crate::wire::{HuntWorld, Lying, PlayerRecord};

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

/// Version of the world format this build writes. 0.11 is 3, like the game's saves.
pub const WORLD_VERSION: u32 = 3;

/// The home rule a new world is made under: 1, the Earth-like home. A world file without one
/// was made under rule 0, the first rule, and keeps its home. It is the game's
/// `universe::HOME_RULE`; the server never reads it, it only hands it to the players.
pub const HOME_RULE: u32 = 1;

/// Returns the version of a world file that has none: the 0.10 format.
fn old_version() -> u32 {
    0
}

/// Everything a server's world file stores. Missing keys take their defaults, so older files
/// still load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct World {
    /// Version of the world format. A file without it is a 0.10 world.
    #[serde(default = "old_version")]
    pub version: u32,
    /// Name of the world, shown in the server list.
    pub name: String,
    /// Seed the whole universe is generated from.
    pub seed: u64,
    /// The home rule the world was made under, see [`HOME_RULE`]: 0 for a file without it.
    #[serde(default)]
    pub home: u32,
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
    /// The carcasses and the stuck spears. It lives in [`HUNT_FILE`], not in the world file, and
    /// holds no item of a player.
    #[serde(skip)]
    pub hunt: HuntWorld,
    /// The things players put on the ground. It lives in [`GROUND_FILE`], not in the world file.
    #[serde(skip)]
    pub ground: Vec<Lying>,
}

impl Default for World {
    /// Returns an empty world named `World` with seed 0.
    fn default() -> Self {
        Self {
            version: WORLD_VERSION,
            name: "World".to_string(),
            seed: 0,
            home: HOME_RULE,
            clock: 0.0,
            changes: Changes::default(),
            atlas: String::new(),
            players: BTreeMap::new(),
            explored: Vec::new(),
            hunt: HuntWorld::default(),
            ground: Vec::new(),
        }
    }
}
