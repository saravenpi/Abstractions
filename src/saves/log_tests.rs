//! Save tests for the log of changes: a file from before the log loads with an empty one, and a
//! log comes back from disk entry for entry.

use std::fs;

use super::model::WorldSave;
use super::store::{list, new_entry, write};
use crate::sim::{Change, Changes, PlayerId};
use crate::universe::{BodyId, Universe};

/// Returns a save of seed 9 whose log holds two first visits, one of them by another player.
fn logged() -> WorldSave {
    let home = Universe::new(9).home();
    let mut changes = Changes::default();
    changes.push(PlayerId::LOCAL, 86_400.0 / 7.0, Change::Visited(home));
    let moon = BodyId { moon: 2, ..home };
    changes.push(PlayerId(3), 0.1 + 0.2, Change::Visited(moon));
    let scratch = std::env::temp_dir().join("abstractions-saves-none");
    WorldSave {
        changes,
        ..new_entry(&scratch, "Logged", 9).save
    }
}

/// A file as 0.8.1 wrote it, with no log, loads with an empty log and everything else equal.
#[test]
fn a_file_without_a_log_loads_with_an_empty_one() {
    let save = WorldSave {
        changes: Changes::default(),
        ..logged()
    };
    let text = toml::to_string(&save).unwrap();
    let old: String = text
        .lines()
        .filter(|line| !line.starts_with("[changes") && !line.starts_with("entries"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(text.contains("[changes]") && !old.contains("changes"));
    let loaded: WorldSave = toml::from_str(&old).unwrap();
    assert!(loaded.changes.entries.is_empty());
    assert_eq!(loaded, save);
}

/// A log comes back from disk with every entry equal, its times bit for bit.
#[test]
fn the_log_round_trips_through_the_file() {
    let dir = std::env::temp_dir().join(format!("abstractions-log-{}", std::process::id()));
    let save = logged();
    write(&dir, "logged", &save).unwrap();
    let loaded = list(&dir).remove(0).save;
    assert_eq!(loaded.changes, save.changes);
    let times = |save: &WorldSave| {
        save.changes
            .entries
            .iter()
            .map(|entry| entry.time.to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(times(&loaded), times(&save));
    assert_eq!(loaded, save);
    let _ = fs::remove_dir_all(&dir);
}
