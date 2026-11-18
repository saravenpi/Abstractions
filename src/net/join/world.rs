//! The world a server's welcome describes, as the saves start path takes it, and the records
//! the server keeps of the player.
//!
//! The world is built in memory and is never written to the local saves folder: a joined
//! session marks it with [`crate::saves::Unsaved`].

use abstractions_net::wire::{PlayerRecord, Welcome};

use crate::computer::{Atlas, Target};
use crate::saves::{Place, SAVE_VERSION, WorldEntry, WorldSave};
use crate::sim::{Change, Changes};
use crate::universe::Universe;

/// Id of a world played on a server. No file ever has it, since it is never written.
pub const SESSION_ID: &str = "session";

/// Seconds of play a player the server already knows counts, so the world is not new to them
/// and they skip the wake-up of a new world.
const RETURNING_SECONDS: f64 = 1.0;

/// Adds every body the log says was visited to `atlas`, so the board computer shows the
/// planets other players found.
pub fn merge(atlas: &mut Atlas, changes: &Changes) {
    for entry in &changes.entries {
        let Change::Visited(body) = entry.change;
        atlas.visit(Target::Body(body));
    }
}

/// Returns the atlas the server keeps as TOML text, or the atlas of a new game when the text
/// is empty or does not read.
pub fn atlas_of(text: &str, universe: &Universe) -> Atlas {
    let read = toml::from_str::<Atlas>(text).ok();
    read.filter(|_| !text.trim().is_empty())
        .unwrap_or_else(|| Atlas::new(universe))
}

/// Returns the world of `welcome`, named `name`: the seed, the universe time, the shared log
/// and atlas, and the place and ship the server kept of the player, if any.
pub fn world_of(welcome: &Welcome, name: &str) -> WorldEntry {
    let universe = Universe::new(welcome.seed);
    let mut atlas = atlas_of(&welcome.atlas, &universe);
    merge(&mut atlas, &welcome.changes);
    let record = welcome.record.clone().unwrap_or_default();
    let place = toml::from_str::<Place>(&record.place).ok();
    let ship = toml::from_str::<toml::Table>(&record.ship).ok();
    let play_seconds = match welcome.record.is_some() {
        true => RETURNING_SECONDS,
        false => 0.0,
    };
    let save = WorldSave {
        name: name.to_string(),
        seed: welcome.seed,
        version: SAVE_VERSION,
        clock_seconds: welcome.clock,
        place: place.filter(|_| !record.place.trim().is_empty()),
        atlas: Some(atlas),
        ship: ship
            .filter(|table| !table.is_empty())
            .map(toml::Value::Table),
        changes: welcome.changes.clone(),
        play_seconds,
        ..WorldSave::default()
    };
    WorldEntry {
        id: SESSION_ID.to_string(),
        save,
    }
}

/// Returns what the server keeps of the player from a captured save: the place and the ship
/// as TOML text, each empty when unknown.
pub fn record_of(save: &WorldSave) -> PlayerRecord {
    let text = |value: Option<String>| value.unwrap_or_default();
    PlayerRecord {
        place: text(save.place.and_then(|place| toml::to_string(&place).ok())),
        ship: text(
            save.ship
                .as_ref()
                .and_then(|ship| toml::to_string(ship).ok()),
        ),
    }
}
