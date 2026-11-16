//! Reading and writing world files in a directory.

use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::log::warn;

use super::id::{slug, unique_id};
use super::model::{WorldEntry, WorldSave};

/// Returns the current time in unix seconds.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Lists the ids of every world file in `dir`, parsable or not.
pub fn ids(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
        .collect()
}

/// Reads one world file, or warns and returns `None` when it cannot be read or parsed.
fn read(dir: &Path, id: &str) -> Option<WorldEntry> {
    let path = dir.join(format!("{id}.toml"));
    let parsed = fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| toml::from_str::<WorldSave>(&text).map_err(|error| error.to_string()));
    match parsed {
        Ok(save) => Some(WorldEntry {
            id: id.to_string(),
            save,
        }),
        Err(error) => {
            warn!("skipping world file {}: {error}", path.display());
            None
        }
    }
}

/// Lists every readable world, most recently played first.
pub fn list(dir: &Path) -> Vec<WorldEntry> {
    let mut worlds: Vec<WorldEntry> = ids(dir).iter().filter_map(|id| read(dir, id)).collect();
    worlds.sort_by(|a, b| {
        b.save
            .last_played
            .cmp(&a.save.last_played)
            .then_with(|| a.id.cmp(&b.id))
    });
    worlds
}

/// Writes a world atomically: a temporary file first, then a rename.
pub fn write(dir: &Path, id: &str, save: &WorldSave) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let text = toml::to_string(save).map_err(io::Error::other)?;
    let temporary = dir.join(format!("{id}.toml.tmp"));
    fs::write(&temporary, text)?;
    fs::rename(&temporary, dir.join(format!("{id}.toml")))
}

/// Removes a world file.
pub fn delete(dir: &Path, id: &str) -> io::Result<()> {
    fs::remove_file(dir.join(format!("{id}.toml")))
}

/// Builds a fresh world with an id that no file in `dir` uses. Nothing is written.
pub fn new_entry(dir: &Path, name: &str, seed: u64) -> WorldEntry {
    let stamp = now();
    WorldEntry {
        id: unique_id(&slug(name), &ids(dir)),
        save: WorldSave {
            name: name.to_string(),
            seed,
            created: stamp,
            last_played: stamp,
            ..WorldSave::default()
        },
    }
}
