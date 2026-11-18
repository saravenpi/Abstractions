//! Save tests for moons: a file written before moons existed still names its planet, and a
//! save that names a moon keeps it.

use std::fs;

use super::model::{Place, WorldSave};
use super::store::{list, new_entry, write};
use crate::computer::{Atlas, Target};
use crate::universe::{BodyId, Universe};

/// Returns a save of seed 9 standing on `body`, with `body` visited and set as the destination.
fn standing(body: BodyId) -> WorldSave {
    let universe = Universe::new(9);
    let mut atlas = Atlas::new(&universe);
    atlas.visit(Target::Body(body));
    atlas.destination = Some(Target::Body(body));
    let scratch = std::env::temp_dir().join("abstractions-saves-none");
    WorldSave {
        place: Some(Place {
            body,
            dir: [0.0, 1.0, 0.0],
            altitude: 2.5,
        }),
        atlas: Some(atlas),
        ..new_entry(&scratch, "Moons", 9).save
    }
}

/// A file as 0.8.0 wrote it, with no `moon` key anywhere, loads and names the same planet in
/// its place, its atlas and its destination.
#[test]
fn a_file_without_moons_names_its_planet() {
    let universe = Universe::new(9);
    let home = universe.home();
    let other = BodyId {
        index: home.index + 1,
        ..home
    };
    let text = toml::to_string(&standing(other)).unwrap();
    let old: String = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("moon ="))
        .map(|line| format!("{line}\n"))
        .collect();
    assert!(text.contains("moon = 0") && !old.contains("moon"));
    let loaded: WorldSave = toml::from_str(&old).unwrap();
    assert_eq!(loaded, standing(other));
    assert_eq!(loaded.body(&universe), other);
    let atlas = loaded.atlas.unwrap();
    assert_eq!(atlas.bodies, [home, other]);
    assert_eq!(atlas.destination, Some(Target::Body(other)));
    assert!(atlas.bodies.iter().all(|body| !body.is_moon()));
}

/// A save that stands on a moon comes back from disk on that moon, with the moon visited,
/// set as the destination, and nothing but its id stored for it.
#[test]
fn a_save_on_a_moon_round_trips() {
    let universe = Universe::new(9);
    let home = universe.home();
    let moon = BodyId {
        index: home.index + 1,
        moon: 2,
        ..home
    };
    let dir = std::env::temp_dir().join(format!("abstractions-moons-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let save = standing(moon);
    write(&dir, "moons", &save).unwrap();
    let loaded = list(&dir).remove(0).save;
    assert_eq!(loaded, save);
    assert_eq!(loaded.body(&universe), moon);
    let atlas = loaded.atlas.unwrap();
    assert!(atlas.visited(Target::Body(moon)) && !atlas.visited(Target::Body(moon.planet())));
    assert_eq!(atlas.destination, Some(Target::Body(moon)));
    let length = |atlas: &Atlas| toml::to_string(atlas).unwrap().len();
    let planet = standing(moon.planet()).atlas.unwrap();
    assert_eq!(
        length(&atlas),
        length(&planet),
        "a moon stores its id and nothing else"
    );
    let _ = fs::remove_dir_all(&dir);
}
