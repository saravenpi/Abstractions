//! Compatibility and sidecar persistence tests for v0.12.2 worlds and protocol 7 data.

use std::fs;

use super::file_tests::scratch;
use super::{
    CHESTS_FILE, DIGS_FILE, FILE, GROUND_FILE, HUNT_FILE, Member, TREES_FILE, WORLD_VERSION, World,
    load_or_create, save,
};
use crate::chest::{CHEST_SLOTS, ChestItem, Chests, ITEM_STACK, PlaceableKind};
use crate::ground::tests::thing;
use crate::ids::BodyId;
use crate::wire::{DigOp, HuntWorld, Rock, TOOL, TreeFell};

/// Returns a sample `Chests` collection containing one chest with one item.
fn sample_chests(body: BodyId) -> Chests {
    let mut chests = Chests::default();
    let prop = chests
        .place(body, [0.0, 1.0, 0.0], 0.5, PlaceableKind::Chest)
        .unwrap();
    chests.set_slot(
        prop.id,
        0,
        Some(ChestItem {
            kind: ITEM_STACK,
            count: 8,
            data: vec![4, 5],
        }),
    );
    chests
}

/// Returns a sample world carrying placed chests, felled trees, terrain digs, and `Member.pack`.
fn world_with_sidecars() -> World {
    let body = BodyId::default();
    let tree = TreeFell {
        body,
        id: "0000000000000009".to_string(),
    };
    let dig = DigOp {
        body,
        dir: [0.0, 1.0, 0.0],
        radius: 1.85,
        depth: 0.48,
    };
    let bob = Member {
        name: "Bob".to_string(),
        pack: "kitted = true\n".to_string(),
        ..Member::default()
    };
    World {
        chests: sample_chests(body),
        trees: vec![tree],
        digs: vec![dig],
        players: [("aa".repeat(32), bob)].into(),
        ..World::default()
    }
}

/// A v0.12.2 server directory (`version = 3`, no `Member.pack`, only `hunt3.bin` and
/// `ground3.bin`) loads cleanly with empty `pack`, `chests`, `trees`, and `digs`.
#[test]
fn loads_v0_12_2_world_without_pack_or_new_sidecars() {
    let dir = scratch("compat-v0-12-2");
    let key = "cd".repeat(32);
    let toml = format!(
        "version = {WORLD_VERSION}\nname = \"Legacy\"\nseed = 998877\nhome = 1\nclock = 4321.5\n\n\
         [players.{key}]\nname = \"Ada\"\nplace = \"body = 1\"\nship = \"s\"\nmarks = \"m\"\n"
    );
    fs::write(dir.join(FILE), toml).unwrap();
    let hunt = HuntWorld {
        rocks: vec![Rock {
            planet: BodyId::default(),
            id: 7,
            grams: 50,
            whole: 100,
        }],
        ..HuntWorld::default()
    };
    let ground = vec![thing(TOOL, 1, 2.5, 0.0)];
    fs::write(dir.join(HUNT_FILE), postcard::to_stdvec(&hunt).unwrap()).unwrap();
    fs::write(dir.join(GROUND_FILE), postcard::to_stdvec(&ground).unwrap()).unwrap();
    let loaded = load_or_create(&dir, "ignored").unwrap();
    assert_eq!((loaded.seed, loaded.clock), (998_877, 4321.5));
    assert_eq!(loaded.players[&key].name, "Ada");
    assert_eq!(loaded.players[&key].pack, "");
    assert_eq!((&loaded.hunt, &loaded.ground), (&hunt, &ground));
    assert!(loaded.chests.props.is_empty() && loaded.trees.is_empty() && loaded.digs.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

/// Placed chests, felled trees, terrain digs, and `Member.pack` round-trip via atomic sidecars.
#[test]
fn round_trips_chests_trees_digs_and_pack() {
    let dir = scratch("compat-sidecars");
    let world = world_with_sidecars();
    save(&dir, &world).unwrap();
    assert!(dir.join(CHESTS_FILE).exists());
    assert!(dir.join(TREES_FILE).exists());
    assert!(dir.join(DIGS_FILE).exists());
    let back = load_or_create(&dir, "ignored").unwrap();
    assert_eq!(back, world);
    assert_eq!(back.chests.props[0].slots.len(), CHEST_SLOTS);
    fs::remove_dir_all(dir).unwrap();
}
