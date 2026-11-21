//! Tests of the ground in the world files: what lies is written beside the world file and read
//! back when the server starts again, and a world from before the ground has none.

use std::fs;

use super::file_tests::scratch;
use super::{GROUND_FILE, World, load_or_create, save};
use crate::ground::tests::thing;
use crate::wire::{HEAP, TOOL};

/// The things on the ground survive a save and a load, as a server that stops and starts again
/// finds them, and a folder without the ground file loads an empty ground.
#[test]
fn the_ground_survives_a_restart() {
    let dir = scratch("ground");
    let world = World {
        ground: vec![thing(TOOL, 1, 1.2, 0.0), thing(HEAP, 2, 4.0, 3.0)],
        ..World::default()
    };
    save(&dir, &world).unwrap();
    assert!(dir.join(GROUND_FILE).exists());
    let back = load_or_create(&dir, "ignored").unwrap();
    assert_eq!(back.ground, world.ground);
    fs::remove_file(dir.join(GROUND_FILE)).unwrap();
    assert!(load_or_create(&dir, "ignored").unwrap().ground.is_empty());
    fs::remove_dir_all(dir).unwrap();
}
