//! Tests of the rules of the map: no border and no rounded corner, no menu of a right click and
//! no second pin system left in its code, and the sentences of its empty states.

use std::fs;
use std::path::Path;

use bevy::math::DVec3;

use super::caption::unexplored;
use super::explored::Explored;
use super::planet::{DRAWING, UNEXPLORED, note_words};
use crate::universe::Universe;

/// Returns the text of every file of the code of `dir` that is not a test, below it.
fn sources(dir: &Path, found: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() && name != "tests" {
            sources(&path, found);
            continue;
        }
        let code = name.ends_with(".rs") && !name.ends_with("_tests.rs");
        if code && !name.starts_with("tests") {
            found.push((name, fs::read_to_string(&path).unwrap()));
        }
    }
}

/// Returns the code of the map and of the computer.
fn code() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    for dir in ["map", "computer"] {
        sources(&root.join(dir), &mut found);
    }
    found
}

/// Nothing in the map or the computer draws a border or a rounded corner.
#[test]
fn the_map_has_no_border_and_no_rounded_corner() {
    let banned = [
        concat!("border", "_radius"),
        concat!("Border", "Radius"),
        concat!("Border", "Color"),
    ];
    for (name, text) in code() {
        for word in banned {
            assert!(!text.contains(word), "{name} has {word}");
        }
    }
}

/// The menu of a right click, the single waypoint and the favourites are gone from the code.
#[test]
fn the_old_menu_and_the_old_pin_systems_are_gone() {
    let gone = [
        concat!("Context", "Menu"),
        concat!("Way", "points"),
        concat!("Favour", "ites"),
        concat!("Back", "Button"),
    ];
    for (name, text) in code() {
        for word in gone {
            assert!(!text.contains(word), "{name} has {word}");
        }
    }
}

/// The map says it is drawing while its picture is not there, that nothing is explored while the
/// picture is black, and nothing otherwise.
#[test]
fn the_empty_map_says_why() {
    assert_eq!(note_words(false, true), DRAWING);
    assert_eq!(note_words(false, false), "Drawing the map...");
    assert_eq!(
        note_words(true, false),
        "Nothing explored here yet. Walk or fly over the ground to chart it."
    );
    assert_eq!(UNEXPLORED, note_words(true, false));
    assert_eq!(note_words(true, true), "");
}

/// The ground under the pointer is explored where the player walked, and not elsewhere.
#[test]
fn the_pointer_knows_the_unexplored_ground() {
    let universe = Universe::new(3);
    let body = universe.home();
    let mut explored = Explored::default();
    assert!(unexplored(&explored, &universe, body, DVec3::X));
    explored.reveal(body, DVec3::X, 0.001, universe.planet(body).radius);
    assert!(!unexplored(&explored, &universe, body, DVec3::X));
    assert!(unexplored(&explored, &universe, body, DVec3::NEG_X));
}
