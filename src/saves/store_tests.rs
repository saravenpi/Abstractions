//! Storage tests on a throwaway directory.

use std::fs;
use std::path::PathBuf;

use bevy::math::IVec3;

use super::model::{Place, SAVE_VERSION, WorldSave};
use super::store::{delete, ids, list, new_entry, write};
use crate::computer::{Atlas, Target};
use crate::universe::{GalaxyId, Universe};
use crate::world::Terrain;

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

/// A world file exactly as v0.5 wrote it.
const V05: &str = r#"name = "Old Isle"
seed = 42
position = [12.5, 3.0, -40.25]
yaw = 0.5
pitch = -0.25
day_phase = 0.4
play_seconds = 5025.5
created = 1700000000
last_played = 1700000500
"#;

/// A v0.5 file loads, keeps its name, seed, play time and dates, and starts at the home spawn.
#[test]
fn a_v05_file_loads_and_starts_at_home() {
    let dir = scratch("v05");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("old-isle.toml"), V05).unwrap();
    let worlds = list(&dir);
    assert_eq!(worlds.len(), 1);
    let mut save = worlds[0].save.clone();
    assert_eq!(
        (save.version, save.position),
        (0, Some([12.5, 3.0, -40.25]))
    );
    save.upgrade();
    assert_eq!((save.name.as_str(), save.seed), ("Old Isle", 42));
    assert_eq!(
        (save.play_seconds, save.created, save.last_played),
        (5025.5, 1_700_000_000, 1_700_000_500)
    );
    assert_eq!(
        (save.version, save.position, save.place),
        (SAVE_VERSION, None, None)
    );
    assert_eq!(
        (save.clock_seconds, &save.atlas, &save.ship),
        (0.0, &None, &None)
    );
    let (universe, terrain) = (Universe::new(42), Terrain::new(42));
    assert_eq!(save.body(&universe), universe.home());
    assert_eq!(save.spawn(&terrain), terrain.find_spawn());
    let _ = fs::remove_dir_all(&dir);
}

/// Returns a save with every v0.6 field set to a value that is awkward to print.
fn voyage() -> WorldSave {
    let universe = Universe::new(9);
    let home = universe.home();
    let mut atlas = Atlas::new(&universe);
    atlas.visit(Target::Galaxy(GalaxyId(IVec3::new(-2, 0, 5))));
    atlas.destination = Some(Target::Body(home));
    let ship = r#"
        frame = "system"
        landed = false
        position = [0.1, -123456789.987654321, 4.0e7]
        velocity = [1e-9, 0.30000000000000004, -7.5]
        orientation = [0.0, 0.7071067811865476, 0.0, 0.7071067811865476]
    "#;
    WorldSave {
        clock_seconds: 86_400.0 / 7.0,
        place: Some(Place {
            body: home,
            dir: [1.0 / 3.0, -(2.0f64.sqrt()) / 2.0, 0.1 + 0.2],
            altitude: -0.000_123_456_789_012_345,
        }),
        atlas: Some(atlas),
        ship: Some(toml::from_str(ship).unwrap()),
        ..new_entry(&scratch("none"), "Voyage", 9).save
    }
}

/// A v0.6 save comes back from disk with every new field equal bit for bit.
#[test]
fn a_v06_save_round_trips_every_field() {
    let dir = scratch("v06");
    let save = voyage();
    write(&dir, "voyage", &save).unwrap();
    let loaded = list(&dir).remove(0).save;
    assert_eq!(loaded, save);
    assert_eq!(loaded.version, SAVE_VERSION);
    assert_eq!(loaded.clock_seconds.to_bits(), save.clock_seconds.to_bits());
    let (old, new) = (save.place.unwrap(), loaded.place.unwrap());
    assert_eq!(new.dir.map(f64::to_bits), old.dir.map(f64::to_bits));
    assert_eq!(new.altitude.to_bits(), old.altitude.to_bits());
    let velocity = |save: &WorldSave| {
        save.ship.as_ref().unwrap()["velocity"][1]
            .as_float()
            .unwrap()
    };
    assert_eq!(
        velocity(&loaded).to_bits(),
        0.300_000_000_000_000_04_f64.to_bits()
    );
    let _ = fs::remove_dir_all(&dir);
}

/// Keys and tables from a newer version are ignored instead of failing the load.
#[test]
fn unknown_fields_do_not_break_the_load() {
    let dir = scratch("future");
    let save = voyage();
    let text = toml::to_string(&save).unwrap();
    let future = format!("crew = 3\n{text}\n[cargo]\nore = [1, 2]\n\n[atlas.rumours]\ncount = 2\n");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("voyage.toml"),
        future.replace("version = 1", "version = 99"),
    )
    .unwrap();
    let loaded = list(&dir).remove(0).save;
    assert_eq!(loaded.version, 99);
    assert_eq!(
        (loaded.place, loaded.atlas, loaded.ship),
        (save.place, save.atlas, save.ship)
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A write that fails leaves the previous file untouched.
#[test]
fn a_failed_write_keeps_the_old_file() {
    let dir = scratch("failed");
    let mut save = voyage();
    write(&dir, "voyage", &save).unwrap();
    let before = fs::read_to_string(dir.join("voyage.toml")).unwrap();
    fs::create_dir_all(dir.join("voyage.toml.tmp")).unwrap();
    save.name = "Lost".to_string();
    assert!(write(&dir, "voyage", &save).is_err());
    assert_eq!(fs::read_to_string(dir.join("voyage.toml")).unwrap(), before);
    assert_eq!(list(&dir).remove(0).save.name, "Voyage");
    let _ = fs::remove_dir_all(&dir);
}
