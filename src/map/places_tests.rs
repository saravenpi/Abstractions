//! Tests of the pins: how they are numbered, capped, removed and renamed, and what the pin key
//! does.

use bevy::input::keyboard::Key;
use bevy::math::DVec3;

use super::keys::{press, reached};
use super::name::edit;
use super::{HERE, MAX, NEAR, Places};
use crate::universe::BodyId;

/// Metres in the radius of the body the tests put pins on.
const RADIUS: f64 = 6.0e6;

/// Returns the point of the body `RADIUS` metres wide that `dir` points at.
fn at(dir: DVec3) -> DVec3 {
    dir.normalize() * RADIUS
}

/// Drops a pin at `dir` on the default body and returns its name.
fn dropped(places: &mut Places, dir: DVec3) -> String {
    let index = places.drop(BodyId::default(), dir).unwrap();
    places.pins[index].name.clone()
}

/// A new pin takes the next free number, also after one was removed.
#[test]
fn pins_are_numbered_from_the_next_free_number() {
    let mut places = Places::default();
    let names: Vec<String> = [DVec3::X, DVec3::Y, DVec3::Z]
        .map(|dir| dropped(&mut places, dir))
        .to_vec();
    assert_eq!(names, ["Pin 1", "Pin 2", "Pin 3"]);
    places.remove(1);
    assert_eq!(dropped(&mut places, DVec3::NEG_X), "Pin 2");
    assert_eq!(dropped(&mut places, DVec3::NEG_Y), "Pin 4");
}

/// The ninth pin is refused, and the key says so.
#[test]
fn the_ninth_pin_is_refused() {
    let mut places = Places::default();
    for step in 0..MAX {
        let dir = DVec3::new(1.0, step as f64, 0.0);
        assert!(places.drop(BodyId::default(), dir).is_some());
    }
    assert_eq!(places.drop(BodyId::default(), DVec3::Z), None);
    let line = press(&mut places, (BodyId::default(), at(DVec3::Z)), RADIUS);
    assert_eq!(
        (line.as_str(), places.pins.len()),
        ("Pins full: remove one", MAX)
    );
}

/// The key twice at the same place drops then removes; twice at far places makes two pins; and a
/// far pin is never removed by the key.
#[test]
fn the_pin_key_removes_near_and_drops_far() {
    let (body, mut places) = (BodyId::default(), Places::default());
    let here = at(DVec3::X);
    assert_eq!(press(&mut places, (body, here), RADIUS), "Pin 1 placed");
    let step = DVec3::Y * 0.5 * NEAR;
    assert_eq!(
        press(&mut places, (body, here + step), RADIUS),
        "Pin 1 removed"
    );
    assert!(places.pins.is_empty());
    press(&mut places, (body, here), RADIUS);
    press(&mut places, (body, here + DVec3::Y * 1000.0), RADIUS);
    assert_eq!(places.pins.len(), 2);
    let other = BodyId { moon: 1, ..body };
    press(&mut places, (other, here), RADIUS);
    assert_eq!(places.pins.len(), 3);
}

/// Removing a pin keeps the active and the selected pin on the pins they were on.
#[test]
fn removing_a_pin_moves_the_active_and_the_selected_pin() {
    let mut places = Places::default();
    for dir in [DVec3::X, DVec3::Y, DVec3::Z] {
        dropped(&mut places, dir);
    }
    places.set_active(Some(2));
    places.selected = Some(1);
    places.remove(0);
    assert_eq!((places.active, places.selected), (Some(1), Some(0)));
    places.remove(0);
    assert_eq!((places.active, places.selected), (Some(0), None));
    places.set_active(Some(7));
    assert_eq!(places.active, None);
    assert_eq!(places.remove(5), None);
}

/// A name is edited with Enter keeping it, Escape giving the old one back, and a blank name
/// is never kept.
#[test]
fn rename_keeps_the_old_name_on_escape() {
    let mut places = Places::default();
    dropped(&mut places, DVec3::X);
    let mut name = String::from("Camp");
    assert!(!edit(&mut name, &Key::Character("s".into()), "Pin 1"));
    assert!(!edit(&mut name, &Key::Backspace, "Pin 1"));
    assert!(edit(&mut name, &Key::Enter, "Pin 1"));
    places.rename(0, &name);
    assert_eq!(places.pins[0].name, "Camp");
    places.rename(0, "   ");
    assert_eq!(places.pins[0].name, "Camp");
    assert!(edit(&mut name, &Key::Escape, "Pin 1"));
    assert_eq!(name, "Pin 1");
    places.rename(0, &name);
    assert_eq!(places.pins[0].name, "Pin 1");
}

/// The active pin is found by its place, and the player is at it within its reach.
#[test]
fn the_active_pin_is_found_and_reached() {
    let mut places = Places::default();
    let index = places.drop(BodyId::default(), DVec3::X).unwrap();
    assert_eq!(places.at(BodyId::default(), at(DVec3::X)), Some(index));
    assert_eq!(places.at(BodyId::default(), at(DVec3::Y)), None);
    places.set_active(Some(index));
    let pin = places.active().unwrap().clone();
    let near = at(DVec3::X) + DVec3::Y * 0.5 * HERE;
    assert!(reached(&pin, (BodyId::default(), near), RADIUS));
    let far = at(DVec3::X) + DVec3::Y * 2.0 * HERE;
    assert!(!reached(&pin, (BodyId::default(), far), RADIUS));
}
