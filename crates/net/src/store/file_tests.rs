//! Tests of the world file: round trip, creation, copies and a crash before the rename.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::file::save_every;
use super::{FILE, Member, World, load_or_create, save};
use crate::ids::BodyId;
use crate::log::{Change, PlayerId};

/// Returns a fresh empty folder named after `test` under the system's temporary folder.
pub(super) fn scratch(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("abstractions-net-{test}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Returns a world with a bit of everything in it.
fn sample() -> World {
    let mut world = World {
        name: "Shared \"isle\"".to_string(),
        seed: 123_456_789_012_345_678,
        clock: 86_400.5,
        atlas: "bodies = []\n".to_string(),
        ..World::default()
    };
    world
        .changes
        .push(PlayerId(3), 12.5, Change::Visited(BodyId::default()));
    let member = Member {
        name: "Ada".to_string(),
        place: "body = 1\n".to_string(),
        ship: "fuel = 0.5\n".to_string(),
        marks: "[[mark]]\n".to_string(),
    };
    world.players.insert("ab".repeat(32), member);
    world
}

/// A world comes back from its file unchanged.
#[test]
fn round_trips_through_the_file() {
    let dir = scratch("round-trip");
    let world = sample();
    save(&dir, &world).unwrap();
    assert_eq!(load_or_create(&dir, "ignored").unwrap(), world);
    fs::remove_dir_all(dir).unwrap();
}

/// An empty folder gets a new world with the given name, written to disk.
#[test]
fn creates_a_world_when_none_exists() {
    let dir = scratch("create").join("data");
    let world = load_or_create(&dir, "Fresh").unwrap();
    assert_eq!(world.name, "Fresh");
    assert!(world.seed < 10u64.pow(18));
    assert!(dir.join(FILE).exists());
    assert_eq!(load_or_create(&dir, "Other").unwrap(), world);
    fs::remove_dir_all(dir.parent().unwrap()).unwrap();
}

/// Many saves keep exactly three copies, the newest being the file before the last save.
#[test]
fn rotation_keeps_three_copies() {
    let dir = scratch("rotation");
    let mut world = sample();
    for clock in 0..6 {
        world.clock = f64::from(clock);
        save_every(&dir, &world, Duration::ZERO).unwrap();
    }
    let copies: Vec<_> = (1..=4)
        .map(|n| dir.join(format!("{FILE}.{n}")).exists())
        .collect();
    assert_eq!(copies, [true, true, true, false]);
    let newest: World =
        toml::from_str(&fs::read_to_string(dir.join(format!("{FILE}.1"))).unwrap()).unwrap();
    assert_eq!(newest.clock, 4.0);
    fs::remove_dir_all(dir).unwrap();
}

/// Saves closer together than the rotation period do not rotate the copies.
#[test]
fn frequent_saves_do_not_rotate() {
    let dir = scratch("no-rotation");
    for _ in 0..4 {
        save(&dir, &sample()).unwrap();
    }
    assert!(dir.join(format!("{FILE}.1")).exists());
    assert!(!dir.join(format!("{FILE}.2")).exists());
    fs::remove_dir_all(dir).unwrap();
}

/// A crash after the temporary file was written and before the rename leaves the old world
/// readable.
#[test]
fn crash_before_rename_keeps_the_old_file() {
    let dir = scratch("crash");
    let world = sample();
    save(&dir, &world).unwrap();
    fs::write(dir.join(format!("{FILE}.tmp")), "name = \"half writ").unwrap();
    assert_eq!(load_or_create(&dir, "ignored").unwrap(), world);
    fs::remove_dir_all(dir).unwrap();
}
