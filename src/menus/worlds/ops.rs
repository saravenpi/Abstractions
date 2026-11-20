//! What can be done to the saved worlds: order, rename, duplicate and delete.
//!
//! Every file operation goes through the public functions of `crate::saves`.

use std::io;
use std::path::Path;

use super::Listed;
use super::when::place_name;
use crate::menus::name::{self, MAX_CHARS};
use crate::saves::{WorldEntry, WorldSave, delete, list, new_entry, write};

/// How the list of worlds is ordered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    /// Most recently played first.
    #[default]
    LastPlayed,
    /// By name, ignoring the letter case.
    Name,
}

impl SortBy {
    /// Returns the other order.
    pub fn next(self) -> Self {
        match self {
            SortBy::LastPlayed => SortBy::Name,
            SortBy::Name => SortBy::LastPlayed,
        }
    }

    /// Returns the words that name the order.
    pub fn label(self) -> &'static str {
        match self {
            SortBy::LastPlayed => "last played",
            SortBy::Name => "name",
        }
    }
}

/// Orders the worlds. Worlds that compare equal are ordered by id, so the order never flickers.
pub fn sort(worlds: &mut [Listed], by: SortBy) {
    worlds.sort_by(|a, b| {
        let (a, b) = (&a.entry, &b.entry);
        let main = match by {
            SortBy::LastPlayed => b.save.last_played.cmp(&a.save.last_played),
            SortBy::Name => a.save.name.to_lowercase().cmp(&b.save.name.to_lowercase()),
        };
        main.then_with(|| a.id.cmp(&b.id))
    });
}

/// Reads every world of `dir` with the place its player is at.
pub fn load(dir: &Path) -> Vec<Listed> {
    let listed = |entry: WorldEntry| Listed {
        place: match entry.save.made_with() {
            Some(version) => format!("Made with {version}"),
            None => place_name(&entry.save),
        },
        entry,
    };
    list(dir).into_iter().map(listed).collect()
}

/// Returns the name of a copy of `name` that none of `taken` has: "name (copy)", then
/// "name (copy 2)" and so on. The name is shortened to leave room for the suffix.
pub fn copy_name(name: &str, taken: &[&str]) -> String {
    let suffixes = (1u32..).map(|n| match n {
        1 => " (copy)".to_string(),
        n => format!(" (copy {n})"),
    });
    let named = suffixes.map(|suffix| {
        let room = MAX_CHARS.saturating_sub(suffix.chars().count());
        let base: String = name.chars().take(room).collect();
        format!("{}{suffix}", base.trim_end())
    });
    let free = |candidate: &String| name::validate(candidate, taken.iter().copied()).is_ok();
    named.take(1000).find(free).unwrap_or_default()
}

/// Gives `world` the typed name and writes its file. The id, which names the file, is kept.
///
/// `others` are the names of the other worlds. Returns the message to show when the name is
/// refused or the file cannot be written; the world is then unchanged.
pub fn rename(
    dir: &Path,
    world: &mut WorldEntry,
    typed: &str,
    others: &[&str],
) -> Result<(), String> {
    let name = name::validate(typed, others.iter().copied())
        .map_err(|refusal| refusal.message().to_string())?;
    let renamed = WorldSave {
        name,
        ..world.save.clone()
    };
    write(dir, &world.id, &renamed).map_err(|error| format!("Could not rename: {error}"))?;
    world.save = renamed;
    Ok(())
}

/// Writes a copy of `world` under a new id and a "(copy)" name, and returns it. The copy keeps
/// the seed, the place, the play time and everything else of the original.
pub fn duplicate(dir: &Path, world: &WorldEntry, taken: &[&str]) -> io::Result<WorldEntry> {
    let name = copy_name(&world.save.name, taken);
    let fresh = new_entry(dir, &name, world.save.seed);
    let copy = WorldEntry {
        id: fresh.id,
        save: WorldSave {
            name,
            created: fresh.save.created,
            ..world.save.clone()
        },
    };
    write(dir, &copy.id, &copy.save)?;
    Ok(copy)
}

/// Deletes the file of the world with this id, and no other.
pub fn remove(dir: &Path, id: &str) -> io::Result<()> {
    delete(dir, id)
}
