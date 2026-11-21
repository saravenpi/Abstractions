//! Tests of the home rule a server's world keeps: an old world file keeps its home, a new world
//! takes the Earth-like one, and an imported game world keeps the rule of its file.

use std::fs;

use super::file_tests::scratch;
use super::{FILE, HOME_RULE, WORLD_VERSION, import, load_or_create, save};

/// A world file of 0.11.0 has no home rule: it loads as rule 0 and keeps it when written back,
/// while a new world is made under [`HOME_RULE`].
#[test]
fn a_served_world_keeps_its_home() {
    let dir = scratch("home-served");
    let text = format!("version = {WORLD_VERSION}\nname = \"Reef\"\nseed = 42\nclock = 9.5\n");
    fs::write(dir.join(FILE), text).unwrap();
    let old = load_or_create(&dir, "ignored").unwrap();
    assert_eq!((old.seed, old.home), (42, 0));
    save(&dir, &old).unwrap();
    assert_eq!(load_or_create(&dir, "ignored").unwrap().home, 0);
    let fresh = scratch("home-fresh");
    assert_eq!(load_or_create(&fresh, "New").unwrap().home, HOME_RULE);
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(fresh).unwrap();
}

/// A game world imported into a server keeps the home rule of its file: rule 0 without one.
#[test]
fn an_imported_world_keeps_its_home() {
    let dir = scratch("home-import");
    let (old, new) = (dir.join("old.toml"), dir.join("new.toml"));
    fs::write(&old, "name = \"Old\"\nseed = 7\nversion = 3\n").unwrap();
    fs::write(
        &new,
        format!("name = \"New\"\nseed = 7\nversion = 3\nhome = {HOME_RULE}\n"),
    )
    .unwrap();
    assert_eq!(import(&old).unwrap().home, 0);
    assert_eq!(import(&new).unwrap().home, HOME_RULE);
    fs::remove_dir_all(dir).unwrap();
}
