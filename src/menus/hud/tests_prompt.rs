//! Tests of the prompt line: the priority order, the course words and the one row.

use bevy::prelude::KeyCode;

use super::course::{Course, words};
use super::prompt::{State, choose};
use crate::computer::{Block, Keys, RouteStatus};
use crate::settings::{KeyLabels, Keymap, Layout};

/// A state with every field set, to take away one at a time.
fn full() -> State {
    State {
        block: Some("Take off first.".to_string()),
        running: Some((KeyCode::KeyG, "Warping to Mars".to_string())),
        ready: Some((KeyCode::KeyG, "Go to Mars".to_string())),
        board: Some((KeyCode::KeyF, "Board")),
        hint: Some((KeyCode::KeyM, "Open the map")),
    }
}

/// A block beats a running course, which beats a course ready, which beats the board key, which
/// beats a hint; with nothing, there is no row.
#[test]
fn the_priority_order_holds() {
    let mut state = full();
    assert_eq!(choose(&state), Some((None, "Take off first.".to_string())));
    state.block = None;
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
    state.board = None;
    assert_eq!(
        choose(&state),
        Some((Some(KeyCode::KeyM), "Open the map".to_string()))
    );
    state.hint = None;
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
    assert_eq!(warp.running.as_deref(), Some("Warping to Mars"));
    let spool = words(&course(RouteStatus::Spooling), true, key);
    assert_eq!(spool.running.as_deref(), Some("Charging the warp drive."));
    let stuck = words(&course(RouteStatus::Blocked(Block::Landed)), true, key);
    assert!(stuck.block.is_some() && stuck.running.is_none());
    let walking = words(&course(RouteStatus::Idle), false, key);
    assert_eq!(
        walking.block.as_deref(),
        Some("Board your ship first: walk to the hatch and press F.")
    );
}

/// The cap of the prompt names the key as the layout prints it: the key of walking forward is
/// W on QWERTY and Z on AZERTY, and a key that prints nothing keeps its name. The sentences of
/// the course name their keys the same way.
#[test]
fn the_cap_names_the_key_as_the_layout_prints_it() {
    let (keys, french) = (Keymap::default(), KeyLabels::of(Layout::Azerty));
    assert_eq!(KeyLabels::default().fill("[{}]", &[keys.forward]), "[W]");
    assert_eq!(french.fill("[{}]", &[keys.forward]), "[Z]");
    assert_eq!(french.fill("[{}]", &[keys.map]), "[,]");
    assert_eq!(french.fill("[{}]", &[keys.jump]), "[Space]");
    let mut learned = KeyLabels::default();
    let typed = bevy::input::keyboard::Key::Character("a".into());
    learned.learn(keys.ship.turn_left, &typed);
    assert_eq!(learned.fill("[{}]", &[keys.ship.turn_left]), "[A]");
    assert_eq!(Keys::of(&keys, &learned).board, "F");
    assert_eq!(Keys::of(&keys, &french).go, "G");
    let stuck = Block::Landed.words(&Keys::of(&keys, &french));
    assert_eq!(stuck, "Take off first: hold Space.");
}
