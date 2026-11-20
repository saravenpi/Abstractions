//! Tests of the explored ground file: it comes back with the world.

use std::fs;

use super::file_tests::scratch;
use super::{EXPLORED_FILE, World, load_or_create, save};
use crate::ids::BodyId;
use crate::leaves::{Leaves, key};

/// The explored ground is saved in its own file and loads back with the world file, while a
/// world without that file loads with no ground.
#[test]
fn explored_ground_comes_back_beside_the_world() {
    let dir = scratch("explored");
    let mut leaves = Leaves::default();
    for morton in [0, 1, 3, 4_000] {
        leaves.insert(key(3, 12, morton));
    }
    let world = World {
        explored: vec![(BodyId::default(), leaves.runs())],
        ..World::default()
    };
    save(&dir, &world).unwrap();
    assert!(dir.join(EXPLORED_FILE).exists());
    assert!(
        !fs::read_to_string(dir.join(super::FILE))
            .unwrap()
            .contains("explored")
    );
    let back = load_or_create(&dir, "x").unwrap();
    assert_eq!(back.explored, world.explored);
    fs::remove_file(dir.join(EXPLORED_FILE)).unwrap();
    assert!(load_or_create(&dir, "x").unwrap().explored.is_empty());
    fs::write(dir.join(EXPLORED_FILE), [0xff; 3]).unwrap();
    assert!(load_or_create(&dir, "x").is_err());
    fs::remove_dir_all(&dir).unwrap();
}
