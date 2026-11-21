//! Tests of the prompt line: the priority order, the course words and the one row.

use bevy::prelude::KeyCode;

use super::course::{CANCEL, Course, words};
use super::prompt::{State, choose, offered};
use crate::computer::{Block, Keys, RouteStatus};
use crate::player::{Offer, Verb};
use crate::settings::Keymap;

/// A state with every field set, to take away one at a time.
fn full() -> State {
    State {
        block: Some("Take off first.".to_string()),
        notice: Some((KeyCode::KeyH, "Flint 1.2 kg".to_string())),
        running: Some((KeyCode::KeyG, "Warping to Mars".to_string())),
        ready: Some((KeyCode::KeyG, "Go to Mars".to_string())),
        act: Some((KeyCode::KeyF, "Board".to_string())),
    }
}

/// A block beats the notice of what was just picked up, which beats a running course, which beats a
/// course ready, which beats the action of a key; with nothing, there is no row.
#[test]
fn the_priority_order_holds() {
    let mut state = full();
    assert_eq!(choose(&state), Some((None, "Take off first.".to_string())));
    state.block = None;
    assert_eq!(
        choose(&state),
        Some((Some(KeyCode::KeyH), "Flint 1.2 kg".to_string()))
    );
    state.notice = None;
    assert_eq!(
        choose(&state),
        Some((Some(KeyCode::KeyG), "Warping to Mars".to_string()))
    );
    state.running = None;
    assert_eq!(
        choose(&state),
        Some((Some(KeyCode::KeyG), "Go to Mars".to_string()))
    );
    state.ready = None;
    assert_eq!(
        choose(&state),
        Some((Some(KeyCode::KeyF), "Board".to_string()))
    );
    state.act = None;
    assert_eq!(choose(&state), None);
}

/// The course words: ready, running, blocked, and on foot a sentence that says how to board.
#[test]
fn the_course_words_follow_the_route() {
    let course = |route| Course {
        name: "Mars".to_string(),
        route,
        gap: None,
    };
    let key = &Keys {
        board: "F".to_string(),
        rise: "Space".to_string(),
        go: "G".to_string(),
    };
    let ready = words(&course(RouteStatus::Idle), true, key);
    assert_eq!(ready.ready.as_deref(), Some("Go to Mars"));
    let warp = words(&course(RouteStatus::InWarp), true, key);
    assert_eq!(warp.running.as_deref(), Some(CANCEL));
    let spool = words(&course(RouteStatus::Spooling), true, key);
    assert_eq!(spool.running.as_deref(), Some(CANCEL));
    let stuck = words(&course(RouteStatus::Blocked(Block::Landed)), true, key);
    assert!(stuck.block.is_some() && stuck.running.is_none());
    let walking = words(&course(RouteStatus::Idle), false, key);
    assert_eq!(
        walking.block.as_deref(),
        Some("Board your ship first: walk to the hatch and press F.")
    );
}

/// Every action a key does names itself and what it acts on, with the key bound to it: the
/// interact key for the five that share it, the pick key for stone, the mend key for a patch.
#[test]
fn every_offered_action_names_its_key_and_its_thing() {
    let mut keys = Keymap::default();
    keys.ship.board = KeyCode::KeyT;
    keys.mend = KeyCode::KeyY;
    let said = |verb, thing| {
        let grams = None;
        offered(Offer { verb, thing, grams }, &keys)
    };
    let row = |key, words: &str| (key, words.to_string());
    assert_eq!(
        said(Verb::Carve, "thigh"),
        row(KeyCode::KeyT, "Cut the thigh")
    );
    assert_eq!(
        said(Verb::Pull, "flint javelin"),
        row(KeyCode::KeyT, "Pull out the flint javelin")
    );
    assert_eq!(
        said(Verb::Take, "flint javelin"),
        row(KeyCode::KeyT, "Take the flint javelin")
    );
    assert_eq!(said(Verb::Board, ""), row(KeyCode::KeyT, "Board"));
    assert_eq!(said(Verb::Leave, ""), row(KeyCode::KeyT, "Leave the ship"));
    assert_eq!(said(Verb::Stone, ""), row(KeyCode::KeyH, "Break off stone"));
    assert_eq!(
        said(Verb::Mend, "Left arm"),
        row(KeyCode::KeyY, "Mend the left arm")
    );
}

/// A take names what it takes and its mass, in kilograms or in grams, on the interact key.
#[test]
fn a_take_names_the_thing_and_its_mass() {
    let mut keys = Keymap::default();
    keys.ship.board = KeyCode::KeyT;
    let flint = Offer {
        verb: Verb::Take,
        thing: "flint",
        grams: Some(1_200),
    };
    let said = |offer| offered(offer, &keys).1;
    assert_eq!(offered(flint, &keys).0, KeyCode::KeyT);
    assert_eq!(said(flint), "Take the flint 1.2 kg");
    let light = Offer {
        grams: Some(300),
        ..flint
    };
    assert_eq!(said(light), "Take the flint 300 g");
    let unweighed = Offer {
        grams: None,
        ..flint
    };
    assert_eq!(said(unweighed), "Take the flint");
}

/// On foot a course set to fly gives the line to an action a key can do now, and keeps it with
/// nothing to do; in the ship it keeps the line.
#[test]
fn an_action_on_foot_beats_the_course() {
    let board = Some("Board the ship to fly: press E.".to_string());
    assert_eq!(super::course::yielded(board.clone(), false, true), None);
    assert_eq!(super::course::yielded(board.clone(), false, false), board);
    assert_eq!(super::course::yielded(board.clone(), true, true), board);
}

/// A cut names the part and what it gives, and breaking stone names its key.
#[test]
fn a_cut_and_a_stone_name_themselves() {
    let mut keys = Keymap::default();
    keys.ship.board = KeyCode::KeyF;
    let cut = Offer {
        verb: Verb::Carve,
        thing: "thigh",
        grams: Some(4_200),
    };
    assert_eq!(
        offered(cut, &keys),
        (KeyCode::KeyF, "Cut the thigh 4.2 kg".to_string())
    );
    let stone = Offer {
        verb: Verb::Stone,
        thing: "",
        grams: None,
    };
    assert_eq!(
        offered(stone, &keys),
        (KeyCode::KeyH, "Break off stone".to_string())
    );
}
