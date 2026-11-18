//! Turning a single-player world of the game into a server's world.

use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

use super::World;
use crate::log::Changes;

/// The keys of the game's world file a server needs. Every other key is ignored, and a
/// missing one takes its default, as in the game.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct GameWorld {
    /// Display name of the world.
    name: Option<String>,
    /// Seed of the universe.
    seed: u64,
    /// Universe time in seconds.
    clock_seconds: f64,
    /// What the player discovered, kept as a table and never interpreted.
    atlas: Option<toml::Table>,
    /// What the player changed, oldest first.
    changes: Changes,
}

/// Returns the server world held in the game's world file at `path`. The atlas comes back as
/// TOML text, or empty when the file has none, so the first player builds a new one.
pub fn import(path: &Path) -> io::Result<World> {
    let text = fs::read_to_string(path)?;
    let game: GameWorld =
        toml::from_str(&text).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let atlas = match game.atlas {
        Some(table) => toml::to_string(&table).map_err(io::Error::other)?,
        None => String::new(),
    };
    Ok(World {
        name: game.name.unwrap_or_else(|| World::default().name),
        seed: game.seed,
        clock: game.clock_seconds,
        changes: game.changes,
        atlas,
        ..World::default()
    })
}
