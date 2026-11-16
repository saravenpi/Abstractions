//! Storage tests on a throwaway directory.

use std::fs;
use std::path::PathBuf;

use super::store::{delete, ids, list, new_entry, write};

/// Returns an empty unique directory under the system temp folder.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("abstractions-saves-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Worlds are written, listed newest first, de-duplicated by id and deleted.
#[test]
fn stores_lists_and_deletes() {
    let dir = scratch("store");
    let mut first = new_entry(&dir, "Isle", 1);
    first.save.last_played = 10;
    write(&dir, &first.id, &first.save).unwrap();
    let mut second = new_entry(&dir, "Isle", 2);
    second.save.last_played = 20;
    assert_eq!(second.id, "isle-2");
    write(&dir, &second.id, &second.save).unwrap();
    let worlds = list(&dir);
    assert_eq!(worlds.len(), 2);
    assert_eq!(worlds[0], second);
    assert!(!dir.join("isle.toml.tmp").exists());
    delete(&dir, &first.id).unwrap();
    assert_eq!(ids(&dir), vec!["isle-2".to_string()]);
    let _ = fs::remove_dir_all(&dir);
}

/// A broken file is skipped, kept on disk, and its id stays reserved.
#[test]
fn broken_files_are_skipped_not_deleted() {
    let dir = scratch("broken");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("isle.toml"), "seed = [").unwrap();
    assert!(list(&dir).is_empty());
    assert!(dir.join("isle.toml").exists());
    assert_eq!(new_entry(&dir, "Isle", 3).id, "isle-2");
    let _ = fs::remove_dir_all(&dir);
}
