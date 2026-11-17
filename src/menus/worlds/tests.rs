//! Worlds menu tests: order, rename, duplicate and delete on a throwaway directory.

use std::fs;
use std::path::PathBuf;

use super::ops::{SortBy, copy_name, duplicate, load, remove, rename, sort};
use super::rows::summary;
use super::{Listed, Worlds};
use crate::menus::name::MAX_CHARS;
use crate::saves::{WorldEntry, list, new_entry, write};

/// Returns an empty unique directory under the system temp folder.
fn scratch(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("abstractions-worlds-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

/// Writes a world named `name` last played at `played` and returns it.
fn world(dir: &std::path::Path, name: &str, seed: u64, played: u64) -> WorldEntry {
    let mut entry = new_entry(dir, name, seed);
    entry.save.last_played = played;
    entry.save.play_seconds = 5025.0;
    write(dir, &entry.id, &entry.save).unwrap();
    entry
}

/// Worlds order by last played, newest first, or by name whatever the case; ties order by id.
#[test]
fn worlds_sort_by_last_played_or_by_name() {
    let dir = scratch("sort");
    for (name, played) in [("beta", 30), ("Alpha", 10), ("gamma", 30), ("Delta", 20)] {
        world(&dir, name, 1, played);
    }
    let mut worlds = load(&dir);
    let ids =
        |worlds: &[Listed]| -> Vec<String> { worlds.iter().map(|w| w.entry.id.clone()).collect() };
    sort(&mut worlds, SortBy::LastPlayed);
    assert_eq!(ids(&worlds), ["beta", "gamma", "delta", "alpha"]);
    sort(&mut worlds, SortBy::Name);
    assert_eq!(ids(&worlds), ["alpha", "beta", "delta", "gamma"]);
    assert_eq!(SortBy::default().next().next(), SortBy::LastPlayed);
    let mut table = Worlds::default();
    table.fill(worlds, Some("delta"));
    assert_eq!(table.selected().unwrap().entry.id, "delta");
    assert_eq!(ids(&table.list), ["beta", "gamma", "delta", "alpha"]);
    table.resort();
    assert_eq!(table.selected().unwrap().entry.id, "delta");
    assert_eq!(table.sort, SortBy::Name);
    assert_eq!(summary(&table), "4 worlds, by name");
    let _ = fs::remove_dir_all(&dir);
}

/// A rename refuses an empty, a taken or a too long name without touching the world.
#[test]
fn rename_refuses_bad_names() {
    let dir = scratch("refuse");
    let mut isle = world(&dir, "Isle", 1, 10);
    world(&dir, "Reef", 2, 20);
    let refused = [
        ("", "A world needs a name"),
        ("   ", "A world needs a name"),
        (" reef ", "Another world has this name"),
        (
            "abcdefghijklmnopqrstuvwxy",
            "A name has at most 24 characters",
        ),
    ];
    for (typed, message) in refused {
        let result = rename(&dir, &mut isle, typed, &["Reef"]);
        assert_eq!(result, Err(message.to_string()));
        assert_eq!(isle.save.name, "Isle");
    }
    let names: Vec<String> = list(&dir).into_iter().map(|w| w.save.name).collect();
    assert_eq!(names, ["Reef", "Isle"]);
    let _ = fs::remove_dir_all(&dir);
}

/// An accepted rename is trimmed and written under the same id, with everything else kept.
#[test]
fn rename_trims_and_keeps_the_id() {
    let dir = scratch("renamed");
    let mut isle = world(&dir, "Isle", 1, 10);
    world(&dir, "Reef", 2, 20);
    assert_eq!(rename(&dir, &mut isle, "  Atoll  ", &["Reef"]), Ok(()));
    assert_eq!(isle.save.name, "Atoll");
    assert_eq!(rename(&dir, &mut isle, "ATOLL", &["Reef"]), Ok(()));
    let on_disk = list(&dir);
    let renamed = on_disk.iter().find(|w| w.id == "isle").unwrap();
    assert_eq!(renamed.save.name, "ATOLL");
    assert_eq!((renamed.save.seed, renamed.save.play_seconds), (1, 5025.0));
    assert_eq!(on_disk.len(), 2);
    let _ = fs::remove_dir_all(&dir);
}

/// A duplicate gets a new id and a "(copy)" name, keeps the seed and the play time, and leaves
/// the original alone. Copy names never clash and never pass the length limit.
#[test]
fn duplicate_makes_a_distinct_world_with_the_same_seed() {
    let dir = scratch("duplicate");
    let isle = world(&dir, "Isle", 77, 10);
    let first = duplicate(&dir, &isle, &["Isle"]).unwrap();
    assert_eq!(
        (first.id.as_str(), first.save.name.as_str()),
        ("isle-copy", "Isle (copy)")
    );
    assert_eq!((first.save.seed, first.save.play_seconds), (77, 5025.0));
    assert_eq!(first.save.last_played, isle.save.last_played);
    let second = duplicate(&dir, &isle, &["Isle", "Isle (copy)"]).unwrap();
    assert_eq!(second.save.name, "Isle (copy 2)");
    assert_ne!(second.id, first.id);
    let on_disk = list(&dir);
    assert_eq!(on_disk.len(), 3);
    assert!(on_disk.iter().all(|w| w.save.seed == 77));
    assert_eq!(on_disk.iter().find(|w| w.id == "isle").unwrap(), &isle);
    let long = "x".repeat(MAX_CHARS);
    let copy = copy_name(&long, &[&long]);
    assert_eq!(copy.chars().count(), MAX_CHARS);
    assert!(copy.ends_with(" (copy)"));
    let _ = fs::remove_dir_all(&dir);
}

/// Deleting a world removes its file and no other.
#[test]
fn delete_removes_only_that_world() {
    let dir = scratch("delete");
    let (isle, reef) = (world(&dir, "Isle", 1, 10), world(&dir, "Reef", 2, 20));
    world(&dir, "Isle", 3, 30);
    remove(&dir, &isle.id).unwrap();
    let left: Vec<String> = list(&dir).into_iter().map(|w| w.id).collect();
    assert_eq!(left, ["isle-2", "reef"]);
    assert!(remove(&dir, &isle.id).is_err());
    assert_eq!(list(&dir).len(), 2);
    assert_eq!(list(&dir).iter().find(|w| w.id == "reef").unwrap(), &reef);
    let _ = fs::remove_dir_all(&dir);
}
