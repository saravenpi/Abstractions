//! Computer tests: typing does not outlive the screen, and the engage key with the screen open
//! engages only while the route is idle, as it does with the screen closed.

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

/// Counts the courses handed to the autopilot.
#[derive(Resource, Default)]
struct Engaged(usize);

/// With the screen open the engage key hands the course to the autopilot only while the route
/// is idle: not once arrived, nor while the ship aligns.
#[test]
fn the_engage_key_waits_for_an_idle_route() {
    let mut app = game("engage-idle");
    app.init_resource::<Engaged>();
    app.add_observer(|_: On<Engage>, mut engaged: ResMut<Engaged>| engaged.0 += 1);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let target = app.world().resource::<Board>().lines[0].target;
    app.world_mut().resource_mut::<Destination>().0 = Some(target);
    for (status, wanted) in [
        (RouteStatus::Arrived, 0),
        (RouteStatus::Aligning(10), 0),
        (RouteStatus::Idle, 1),
    ] {
        *app.world_mut().resource_mut::<RouteStatus>() = status;
        tap(&mut app, KeyCode::KeyG, None);
        assert_eq!(app.world().resource::<Engaged>().0, wanted, "{status:?}");
    }
}
