//! Tests of worlds made by an older release: they list as old and are never rewritten.

use std::fs;

use super::store::{list, new_entry, write};
use super::store_tests::{V05, scratch};

/// A v0.5 file loads as an old world, keeps its name, seed, play time and dates untouched, and
/// is never opened by this game.
#[test]
fn a_v05_file_loads_as_an_old_world() {
    let dir = scratch("v05");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("old-isle.toml"), V05).unwrap();
    let worlds = list(&dir);
    assert_eq!(worlds.len(), 1);
    let save = &worlds[0].save;
    assert_eq!(
        (save.version, save.position),
        (0, Some([12.5, 3.0, -40.25]))
    );
    assert_eq!((save.name.as_str(), save.seed), ("Old Isle", 42));
    assert_eq!(
        (save.play_seconds, save.created, save.last_played),
        (5025.5, 1_700_000_000, 1_700_000_500)
    );
    assert_eq!(
        (save.game_version.as_str(), save.made_with()),
        ("", Some("0.10"))
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A version 2 file, as 0.10 wrote it, loads as an old world and stays byte for byte as it was
/// after the list is read, while a file of this version is not old and names this game.
#[test]
fn a_version_2_file_is_an_old_world_and_a_new_one_is_not() {
    let dir = scratch("v2");
    fs::create_dir_all(&dir).unwrap();
    let text = "name = \"Reef\"\nseed = 5\nversion = 2\n";
    fs::write(dir.join("reef.toml"), text).unwrap();
    let old = list(&dir).remove(0).save;
    assert_eq!((old.version, old.made_with()), (2, Some("0.10")));
    assert_eq!(fs::read_to_string(dir.join("reef.toml")).unwrap(), text);
    let fresh = new_entry(&dir, "Fresh", 6).save;
    assert_eq!(fresh.made_with(), None);
    assert_eq!(fresh.game_version, env!("CARGO_PKG_VERSION"));
    write(&dir, "fresh", &fresh).unwrap();
    let back = list(&dir).into_iter().find(|world| world.id == "fresh");
    assert_eq!(back.unwrap().save.made_with(), None);
    let _ = fs::remove_dir_all(&dir);
}
