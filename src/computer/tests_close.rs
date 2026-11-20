//! Computer tests: typing does not outlive the screen, and the Go key with the screen open
//! starts an idle route and stops one that is flown.

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

/// Counts the courses handed to the autopilot and the routes stopped.
#[derive(Resource, Default)]
struct Calls {
    /// Courses handed over.
    engaged: usize,
    /// Routes stopped.
    cancelled: usize,
}

/// Returns the index of the first row of the table that is not where the player is.
pub(super) fn elsewhere(app: &App) -> usize {
    let board = app.world().resource::<Board>();
    let away = |line: &Line| !board.here.contains(&Some(line.target));
    board.lines.iter().position(away).expect("a place to go to")
}

/// With the screen open, the Go key stops a route that is flown, does nothing while the drive
/// is blocked and sends the ship to the course while the route is idle.
#[test]
fn the_go_key_stops_a_route_and_starts_an_idle_one() {
    let mut app = game("go-key");
    app.init_resource::<Calls>();
    app.add_observer(|_: On<Engage>, mut calls: ResMut<Calls>| calls.engaged += 1);
    app.add_observer(|_: On<Cancel>, mut calls: ResMut<Calls>| calls.cancelled += 1);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let at = elsewhere(&app);
    let target = app.world().resource::<Board>().lines[at].target;
    app.world_mut().resource_mut::<Destination>().0 = Some(target);
    let landed = RouteStatus::Blocked(Block::Landed);
    for (status, wanted) in [
        (RouteStatus::Aligning(10), (0, 1)),
        (landed, (0, 0)),
        (RouteStatus::Idle, (1, 0)),
    ] {
        *app.world_mut().resource_mut::<RouteStatus>() = status;
        *app.world_mut().resource_mut::<Calls>() = Calls::default();
        tap(&mut app, KeyCode::KeyG, None);
        let calls = app.world().resource::<Calls>();
        assert_eq!((calls.engaged, calls.cancelled), wanted, "{status:?}");
    }
    run(&mut app, 3);
    assert!(!open(&app));
}

/// A player on foot who presses Enter on a row gets no course and no route: the card says to
/// board the ship first.
#[test]
fn go_does_nothing_on_foot() {
    let mut app = game("go-foot");
    app.init_resource::<Calls>();
    app.add_observer(|_: On<Engage>, mut calls: ResMut<Calls>| calls.engaged += 1);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let at = elsewhere(&app);
    app.world_mut().resource_mut::<Board>().step(at as isize);
    app.world_mut()
        .resource_mut::<crate::space::Location>()
        .travel = crate::space::Travel::OnFoot;
    tap(&mut app, KeyCode::Enter, None);
    assert_eq!(app.world().resource::<Calls>().engaged, 0);
    assert_eq!(app.world().resource::<Destination>().0, None);
}
