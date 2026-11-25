//! Tests of importing a single-player world written in the game's format.

use std::fs;

use glam::IVec3;

use super::file_tests::scratch;
use super::import;
use crate::ids::{BodyId, GalaxyId, StarId};
use crate::log::{Change, PlayerId};

/// A world file as the game writes it, keys and nesting taken from `src/saves/model.rs`.
const GAME_FILE: &str = r#"
name = "Isle one"
seed = 424242
version = 2
yaw = 0.5
pitch = -0.25
day_phase = 0.3
play_seconds = 5025.5
created = 1700000000
last_played = 1700000500
clock_seconds = 98765.25

[place]
dir = [0.0, 1.0, 0.0]
altitude = 2.5

[place.body]
index = 2
moon = 0

[place.body.star]
galaxy = [0, 0, 0]
sector = [1, -2, 3]
index = 4

[atlas]
galaxies = [[0, 0, 0]]
systems = [{ galaxy = [0, 0, 0], sector = [1, -2, 3], index = 4 }]
bodies = [{ star = { galaxy = [0, 0, 0], sector = [1, -2, 3], index = 4 }, index = 2, moon = 0 }]
walked = [{ star = { galaxy = [0, 0, 0], sector = [1, -2, 3], index = 4 }, index = 2 }]

[atlas.destination.Star]
galaxy = [0, 0, 0]
sector = [1, -2, 3]
index = 5

[ship]
fuel = 0.75

[[changes.entries]]
player = 0
time = 120.5

[changes.entries.change.Visited]
index = 3
moon = 1

[changes.entries.change.Visited.star]
galaxy = [0, 0, 0]
sector = [1, -2, 3]
index = 4
"#;

/// Returns the body the fixture's log names.
fn visited() -> BodyId {
    let star = StarId {
        galaxy: GalaxyId(IVec3::ZERO),
        sector: IVec3::new(1, -2, 3),
        index: 4,
    };
    BodyId {
        star,
        index: 3,
        moon: 1,
    }
}

/// Name, seed, clock and log come across, and the atlas comes back as the same table.
#[test]
fn imports_a_game_world() {
    let dir = scratch("import");
    let path = dir.join("isle.toml");
    fs::write(&path, GAME_FILE).unwrap();
    let world = import(&path).unwrap();
    assert_eq!(world.name, "Isle one");
    assert_eq!(world.seed, 424_242);
    assert_eq!(world.clock, 98_765.25);
    assert_eq!(world.changes.entries.len(), 1);
    let entry = &world.changes.entries[0];
    assert_eq!((entry.player, entry.time), (PlayerId(0), 120.5));
    assert_eq!(entry.change, Change::Visited(visited()));
    let atlas: toml::Table = toml::from_str(&world.atlas).unwrap();
    let original: toml::Table = toml::from_str(GAME_FILE).unwrap();
    assert_eq!(Some(&toml::Value::Table(atlas)), original.get("atlas"));
    assert!(world.players.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

/// A file from before the clock, the atlas and the log still imports.
#[test]
fn imports_an_old_world() {
    let dir = scratch("import-old");
    let path = dir.join("old.toml");
    fs::write(&path, "name = \"Old\"\nseed = 7\n").unwrap();
    let world = import(&path).unwrap();
    assert_eq!(
        (world.name.as_str(), world.seed, world.clock),
        ("Old", 7, 0.0)
    );
    assert!(world.atlas.is_empty());
    assert!(world.changes.entries.is_empty());
    fs::remove_dir_all(dir).unwrap();
}

/// A file that is not TOML is refused, not taken as an empty world.
#[test]
fn refuses_a_broken_file() {
    let dir = scratch("import-broken");
    let path = dir.join("broken.toml");
    fs::write(&path, "name = ").unwrap();
    assert!(import(&path).is_err());
    fs::remove_dir_all(dir).unwrap();
}

/// Felled trees, terrain digs, and placed chests import cleanly while partial tree hits are skipped.
#[test]
fn imports_felled_digs_and_chests() {
    let dir = scratch("import-world-props");
    let path = dir.join("world.toml");
    let text = format!(
        "felled = [\"0000000000000010:3\", \"0000000000000020:1\", \"0000000000000010:3\"]\n\
         {GAME_FILE}\n\
         [[digs]]\n\
         dir = [0.0, 1.0, 0.0]\n\
         radius = 1.85\n\
         depth = 0.48\n\
         [digs.planet]\n\
         index = 2\n\
         moon = 0\n\
         [digs.planet.star]\n\
         galaxy = [0, 0, 0]\n\
         sector = [1, -2, 3]\n\
         index = 4\n"
    );
    fs::write(&path, text).unwrap();
    let world = import(&path).unwrap();
    assert_eq!(world.trees.len(), 1);
    assert_eq!(world.trees[0].id, "0000000000000010");
    assert_eq!(world.trees[0].body.index, 2);
    assert_eq!(world.digs.len(), 1);
    assert_eq!(world.digs[0].body.index, 2);
    assert!(world.chests.props.is_empty());
    fs::remove_dir_all(dir).unwrap();
}
