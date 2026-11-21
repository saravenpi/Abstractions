//! Tests of the home rule a save keeps: a world saved before the home had to be Earth-like
//! loads onto the home it had, and a new world is made under the Earth-like rule.

use std::fs;

use super::model::WorldSave;
use super::store::{list, new_entry, write};
use super::store_tests::scratch;
use crate::universe::{HOME_RULE, Universe};

/// Returns a world file as 0.11.0 wrote it for `seed`, with no home rule in it.
fn first(seed: u64) -> String {
    format!("name = \"Reef\"\nseed = {seed}\nversion = 3\ngame_version = \"0.11.0\"\n")
}

/// A file of 0.11.0 reads as rule 0 and starts on the home that rule gives, the home it had
/// when it was saved, for 20 seeds, while a new world of the same seed starts on the home of
/// the Earth-like rule. At least one of those seeds has a different home under each rule, so
/// without the rule in the save that world would wake on another planet.
#[test]
fn a_saved_world_keeps_its_home() {
    let mut moved = 0;
    for seed in 0..20 {
        let old: WorldSave = toml::from_str(&first(seed)).unwrap();
        assert_eq!(old.home, 0, "seed {seed}");
        assert_eq!(old.made_with(), None, "seed {seed}");
        let kept = Universe::under(seed, 0);
        assert_eq!(old.universe(), kept, "seed {seed}");
        assert_eq!(old.body(&old.universe()), kept.home(), "seed {seed}");
        let fresh = WorldSave {
            seed,
            ..WorldSave::default()
        };
        assert_eq!(fresh.universe(), Universe::new(seed), "seed {seed}");
        moved += usize::from(fresh.universe().home() != kept.home());
    }
    println!("of 20 seeds, {moved} have another home under the Earth-like rule");
    assert!(moved > 0);
}

/// A new world is made under [`HOME_RULE`] and keeps it through the file, and an old file
/// keeps its rule 0 when it is written back after play.
#[test]
fn the_rule_survives_the_file() {
    let dir = scratch("home-rule");
    fs::create_dir_all(&dir).unwrap();
    let fresh = new_entry(&dir, "Fresh", 6).save;
    assert_eq!(fresh.home, HOME_RULE);
    write(&dir, "fresh", &fresh).unwrap();
    fs::write(dir.join("reef.toml"), first(5)).unwrap();
    let mut old = list(&dir)
        .into_iter()
        .find(|w| w.id == "reef")
        .unwrap()
        .save;
    old.play_seconds = 60.0;
    write(&dir, "reef", &old).unwrap();
    let worlds = list(&dir);
    let home = |id: &str| worlds.iter().find(|w| w.id == id).map(|w| w.save.home);
    assert_eq!((home("fresh"), home("reef")), (Some(HOME_RULE), Some(0)));
    let _ = fs::remove_dir_all(&dir);
}

/// The server keeps the home rule under the same number as the game, so a hosted or served
/// world hands its players the home the game would pick.
#[test]
fn the_server_counts_rules_as_the_game_does() {
    assert_eq!(abstractions_net::store::HOME_RULE, HOME_RULE);
    let world = abstractions_net::store::World::default();
    assert_eq!(world.home, HOME_RULE);
}
