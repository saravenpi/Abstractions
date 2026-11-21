//! Tests of the words of the orbital cruise on the prompt line: each says what the cruise does
//! with the key that does it, and a running cruise takes the line from the course.

use super::cruise::{Said, clock, lines};
use crate::settings::Keymap;
use crate::ship::{Note, Reading, Regime};

/// Times to arrival read as minutes and seconds under an hour.
#[test]
fn times_read_in_minutes_and_seconds() {
    assert_eq!(clock(45.2), "45 s");
    assert_eq!(clock(80.0), "1 min 20 s");
    assert_eq!(clock(7200.0), "2 h");
}

/// A cruise to home says how far home is, how long it takes and where the glide begins, with
/// the warp key that stops it, and takes the line.
#[test]
fn a_cruise_home_says_how_far_and_how_long() {
    let keys = Keymap::default();
    let said = Said {
        regime: Regime::Cruise,
        reading: Some(Reading {
            arc: 2_340_000.0,
            eta: 80.0,
            descent: Some(2_210_000.0),
        }),
        ..Said::default()
    };
    let shown = lines(&said, &keys);
    let (key, words) = shown.running.expect("a running line");
    assert_eq!(key, keys.ship.warp);
    assert!(words.starts_with("Home 2"), "{words}");
    assert!(
        words.contains("1 min 20 s") && words.contains("glide in"),
        "{words}"
    );
    assert!(shown.taken);
}

/// Above the air a cruise is ready on the warp key, and over the planet of the base the line
/// points to the map, where Fly home is, after any course ready to go.
#[test]
fn a_cruise_ready_shows_its_key() {
    let keys = Keymap::default();
    let ready = Said {
        ready: true,
        ..Said::default()
    };
    let away = lines(&ready, &keys);
    assert_eq!(away.ready.map(|(key, _)| key), Some(keys.ship.warp));
    assert!(!away.taken);
    let home = lines(
        &Said {
            home: true,
            ..ready
        },
        &keys,
    );
    assert_eq!(home.ready.map(|(key, _)| key), Some(keys.map));
    assert_eq!(lines(&Said::default(), &keys).ready, None);
}

/// Why a cruise ended is said once it did: the floor as a sentence, the end of the glide with the
/// key to sink and land.
#[test]
fn the_end_of_a_cruise_is_said() {
    let keys = Keymap::default();
    let floor = lines(
        &Said {
            note: Some(Note::Floor),
            ..Said::default()
        },
        &keys,
    );
    assert!(floor.block.is_some() && floor.taken);
    let landed = lines(
        &Said {
            note: Some(Note::Arrived),
            ..Said::default()
        },
        &keys,
    );
    assert_eq!(landed.ready.map(|(key, _)| key), Some(keys.ship.sink));
}
