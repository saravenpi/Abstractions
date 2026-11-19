//! Computer tests: typing does not outlive the screen.

use bevy::prelude::*;

use super::tests_rig::{game, run, tap};
use super::*;

/// Returns true while the screen is open.
fn open(app: &App) -> bool {
    *app.world().resource::<State<Computer>>().get() == Computer::Open
}

/// Closing the screen in the middle of a search ends the search and clears it, so the keys of
/// the game work again and Escape closes the screen once it opens again.
#[test]
fn closing_the_screen_ends_typing() {
    let mut app = game("close-typing");
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    tap(&mut app, KeyCode::Slash, None);
    tap(&mut app, KeyCode::KeyK, Some("k"));
    assert_eq!(*app.world().resource::<Typing>(), Typing::Search);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 3);
    assert!(!open(&app));
    assert!(app.world().resource::<Typing>().is_off());
    assert!(app.world().resource::<Board>().filter.is_empty());
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    assert!(open(&app));
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(!open(&app));
}
