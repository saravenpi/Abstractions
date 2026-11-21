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
/// interact key for the four that share it, the pick key for stone, the mend key for a patch.
#[test]
fn every_offered_action_names_its_key_and_its_thing() {
    let mut keys = Keymap::default();
    keys.ship.board = KeyCode::KeyT;
    keys.mend = KeyCode::KeyY;
    let said = |verb, thing| offered(Offer { verb, thing }, &keys);
    let row = |key, words: &str| (key, words.to_string());
    assert_eq!(
        said(Verb::Carve, "thigh"),
        row(KeyCode::KeyT, "Cut the thigh")
    );
    assert_eq!(
        said(Verb::Pull, "spear"),
        row(KeyCode::KeyT, "Pull the spear")
    );
    assert_eq!(said(Verb::Board, ""), row(KeyCode::KeyT, "Board"));
    assert_eq!(said(Verb::Leave, ""), row(KeyCode::KeyT, "Leave the ship"));
    assert_eq!(said(Verb::Stone, ""), row(KeyCode::KeyH, "Break stone off"));
    assert_eq!(
        said(Verb::Mend, "Left arm"),
        row(KeyCode::KeyY, "Mend the left arm")
    );
}
