//! Tests of the check of a stored place: what stays, what is grounded and what is forgotten.

use super::place::{HIGHEST, LOWEST, mend, mend_all, repaired};
use super::{Member, World};

/// Returns the text of a place `altitude` metres up, as the game writes it.
fn place(altitude: &str) -> String {
    format!(
        "dir = [0.6, 0.0, -0.8]\naltitude = {altitude}\n\n[body]\nindex = 0\nmoon = 0\n\n\
         [body.star]\ngalaxy = [0, 0, 0]\nsector = [35, 0, -44]\nindex = 0\n"
    )
}

/// Returns the height and the direction of the place `text`.
fn read(text: &str) -> (f64, Vec<f64>) {
    let table: toml::Table = toml::from_str(text).expect("a place");
    let dir = table["dir"].as_array().expect("a direction");
    let dir = dir.iter().map(|part| part.as_float().expect("a number"));
    (
        table["altitude"].as_float().expect("a height"),
        dir.collect(),
    )
}

/// An unknown place and a place within the range stay as they are, the air included: the
/// place the owner was stuck at, 6560 m up, is a valid place.
#[test]
fn a_sound_place_stays() {
    assert_eq!(repaired(""), None);
    assert_eq!(repaired("  \n"), None);
    for altitude in ["2.0", "6560.18", "-300.0", "2", "2.0e6", "4.0e11"] {
        assert_eq!(repaired(&place(altitude)), None, "{altitude}");
    }
    assert_eq!(repaired(&place(&format!("{LOWEST:?}"))), None);
    assert_eq!(repaired(&place(&format!("{HIGHEST:?}"))), None);
}

/// A height that is not a number or out of range becomes the lowest one, which the game reads
/// as the ground, and the direction and the body are kept. The repaired place is sound.
#[test]
fn a_bad_height_is_grounded_in_the_same_direction() {
    for altitude in ["nan", "inf", "-inf", "1.0e18", "-1.0e9"] {
        let fixed = repaired(&place(altitude)).expect(altitude);
        assert_eq!(read(&fixed), (LOWEST, vec![0.6, 0.0, -0.8]), "{altitude}");
        assert!(fixed.contains("sector = [35, 0, -44]"), "{fixed}");
        assert_eq!(repaired(&fixed), None);
    }
}

/// A text that is no place, a place without a body and a direction without a length or with a
/// part that is not a number are forgotten.
#[test]
fn a_place_that_does_not_read_is_forgotten() {
    let no_dir = place("2.0").replace("[0.6, 0.0, -0.8]", "[0.0, 0.0, 0.0]");
    let nan_dir = place("2.0").replace("[0.6, 0.0, -0.8]", "[nan, 0.0, 1.0]");
    let no_body = "dir = [0.0, 1.0, 0.0]\naltitude = 2.0\n";
    for text in ["p", "body = 1\n", no_body, &no_dir, &nan_dir] {
        assert_eq!(repaired(text), Some(String::new()), "{text}");
    }
}

/// Mending a world repairs every bad place and says so once for each, with the name of the
/// player, and leaves the good ones alone.
#[test]
fn mending_a_world_logs_each_repair() {
    let member = |name: &str, place: String| Member {
        name: name.to_string(),
        place,
        ..Member::default()
    };
    let players = [
        ("a".to_string(), member("Ada", place("inf"))),
        ("b".to_string(), member("Bob", place("2.0"))),
        ("c".to_string(), member("Cid", "p".to_string())),
    ];
    let mut world = World {
        players: players.into(),
        ..World::default()
    };
    let lines = mend_all(&mut world);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("The stored place of Ada is not valid, the player is put on"));
    assert!(lines[1].starts_with("The stored place of Cid is not valid, it is forgotten"));
    assert_eq!(read(&world.players["a"].place).0, LOWEST);
    assert_eq!(world.players["b"].place, place("2.0"));
    assert_eq!(world.players["c"].place, "");
    assert!(mend_all(&mut world).is_empty());
    let mut bob = world.players["b"].clone();
    assert_eq!(mend(&mut bob), None);
}
