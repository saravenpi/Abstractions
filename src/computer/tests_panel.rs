//! Computer tests: the measures of the side panel, the words of the footer, the buttons of
//! the card, and the open panel leaving the game running.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::course::{NO_COURSE, arrival, course, progress};
use super::layout::{FACTS, Measures, RESULTS, ROW, SIDE, SMALLEST, fixed, measure};
use super::tests_rig::{game, run, tap};
use super::view::{Panel, Slot};
use super::*;
use crate::map::{Choice, Deed, offer, reason};
use crate::menus::Menu;
use crate::screens::{in_control, playing};

/// At every window size the side panel keeps its width, the map is the largest square beside
/// it, and the search results take what the panel leaves within their fewest and most rows.
/// At the smallest window the search, one result, the top of the card with its buttons and
/// the lists always fit, and only the facts of the card are cut; at 1280 by 720 the whole
/// card fits with nothing listed.
#[test]
fn the_screen_fits_every_window() {
    for height in [540.0, 600.0, 720.0, 900.0, 1080.0, 1200.0, 1440.0, 2160.0] {
        for width in [960.0, 1280.0, 1920.0, 2560.0, 3840.0] {
            let window = Vec2::new(width, height);
            let fit = measure(window, (0, 0));
            assert!(
                fit.map <= height && fit.map + fit.side <= width,
                "{window}: {fit:?}"
            );
            assert!(
                fit.map == height || fit.map == width - SIDE,
                "{window}: {fit:?}"
            );
            assert!((RESULTS.0..=RESULTS.1).contains(&fit.rows), "{window}");
        }
    }
    let smallest = Measures {
        side: SIDE,
        map: 540.0,
        rows: RESULTS.0,
    };
    assert_eq!(measure(SMALLEST, (0, 0)), smallest);
    for lists in [(0, 0), (2, 1), (5, 0), (3, 3), (1, 6)] {
        let fit = measure(SMALLEST, lists);
        let used = fixed(lists) - FACTS + fit.rows as f32 * ROW;
        assert!(used <= SMALLEST.y, "{lists:?}: {used}");
    }
    let usual = measure(Vec2::new(1280.0, 720.0), (0, 0));
    assert!(fixed((0, 0)) + usual.rows as f32 * ROW <= 720.0);
}

/// The course line gives a time to arrival only while the ship moves.
#[test]
fn the_footer_says_the_right_words() {
    let idle = RouteStatus::Idle;
    assert_eq!(arrival(6000.0, 100.0).as_deref(), Some("about 1 min"));
    assert_eq!(arrival(6000.0, 0.5), None);
    assert_eq!(
        course(Some(("Kodya", 1500.0)), 0.0, idle),
        "Course Kodya - 1.50 km - idle"
    );
    let moving = course(Some(("Kodya", 720_000.0)), 100.0, idle);
    assert_eq!(moving, "Course Kodya - 720 km - about 2 h 0 min - idle");
    assert_eq!(course(None, 100.0, idle), NO_COURSE);
}

/// The action line offers the engage key only while the autopilot can take the course, and
/// otherwise says why not or how far along the route is.
#[test]
fn the_action_line_says_what_the_route_can_do() {
    let idle = RouteStatus::Idle;
    let engage = (true, "Engage autopilot to Kodya b".to_string());
    assert_eq!(action("Kodya b", idle), engage);
    let inside = Block::Inside {
        star: false,
        lock: 3,
        tenths: 14,
    };
    let blocked = action("Kodya b", RouteStatus::Blocked(inside));
    assert_eq!(
        blocked,
        (
            false,
            "Inside 3 planet radii (now 1.4): fly out first".to_string()
        )
    );
    assert_eq!(
        action("Kodya b", RouteStatus::Aligning(45)).1,
        "Aligning 45%"
    );
    assert!(!action("Kodya b", RouteStatus::Blocked(Block::Landed)).0);
    assert_eq!(
        action("Kodya b", RouteStatus::Arrived).1,
        "Arrived at Kodya b"
    );
}

/// The progress line under the map says how far along the route is while it is flown, and
/// nothing while the route waits, is blocked or has arrived.
#[test]
fn the_progress_line_shows_only_a_route_in_flight() {
    assert_eq!(progress("Kodya", RouteStatus::Aligning(40)), "Aligning 40%");
    assert_eq!(progress("Kodya", RouteStatus::InWarp), "In warp to Kodya");
    let quiet = [
        RouteStatus::Idle,
        RouteStatus::Blocked(Block::Landed),
        RouteStatus::Arrived,
    ];
    assert!(
        quiet
            .iter()
            .all(|status| progress("Kodya", *status).is_empty())
    );
}

/// The card of a place offers to set the course there while it is not the course, and to
/// engage while the route to it is idle and to clear it while it is, with the other choices
/// of the menu of a right click after them. Only the card of the course says why the drive
/// cannot engage.
#[test]
fn the_card_offers_the_buttons_of_the_course() {
    let universe = crate::universe::Universe::new(7);
    let (planet, star) = (
        Target::Body(universe.home()),
        Target::Star(universe.home().star),
    );
    let (idle, landed) = (RouteStatus::Idle, RouteStatus::Blocked(Block::Landed));
    let menu = |choice| Deed::Choice(choice);
    let others = [
        menu(Choice::Focus),
        menu(Choice::Waypoint),
        menu(Choice::Favourite),
    ];
    let free = offer(planet, Some(star), idle);
    assert_eq!(free[0], menu(Choice::Course));
    assert_eq!(free[1..], others);
    assert_eq!(
        offer(planet, Some(planet), idle)[..2],
        [Deed::Engage, Deed::Clear]
    );
    assert_eq!(
        offer(planet, Some(planet), landed)[..2],
        [Deed::Clear, others[0]]
    );
    assert_eq!(offer(star, None, idle), [menu(Choice::Course), others[0]]);
    assert_eq!(
        reason(planet, Some(planet), landed),
        "Landed: take off first"
    );
    assert_eq!(reason(star, Some(planet), landed), "");
    assert_eq!(reason(planet, Some(planet), idle), "");
}

/// Opening the map leaves the game running: physics and the game clock are not paused and
/// play goes on, while the keys and the mouse leave the ship. Escape closes it without
/// opening the pause menu, and the pause menu closes it.
#[test]
fn the_open_panel_leaves_the_game_running() {
    let mut app = game("running");
    let open = |app: &App| *app.world().resource::<State<Computer>>().get() == Computer::Open;
    let menu = |app: &App| *app.world().resource::<State<Menu>>().get();
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 30);
    assert!(open(&app) && menu(&app) == Menu::None);
    assert!(!app.world().resource::<Time<Physics>>().is_paused());
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    assert!(app.world_mut().run_system_once(playing).unwrap());
    assert!(!app.world_mut().run_system_once(in_control).unwrap());
    let mut panels = app.world_mut().query_filtered::<&Node, With<Panel>>();
    let panel = panels.single(app.world()).unwrap();
    assert_eq!((panel.width, panel.height), (percent(100), percent(100)));
    let mut slots = app.world_mut().query::<(&Slot, &Node)>();
    let drawn = slots
        .iter(app.world())
        .filter(|(_, node)| node.display != Display::None);
    assert!(drawn.count() <= measure(SMALLEST, (0, 0)).rows);
    tap(&mut app, KeyCode::Escape, None);
    run(&mut app, 2);
    assert!(!open(&app) && menu(&app) == Menu::None);
    tap(&mut app, KeyCode::Tab, None);
    assert!(open(&app));
    app.world_mut()
        .resource_mut::<NextState<Menu>>()
        .set(Menu::Pause);
    run(&mut app, 3);
    assert!(!open(&app) && menu(&app) == Menu::Pause);
}

/// The map key opens the map anywhere in the world, the key of the board computer only while
/// seated, either closes it, and a menu or leaving the world closes it.
#[test]
fn the_map_opens_on_foot_and_in_the_seat() {
    use super::open::next_state;
    let (on_foot, seated, away) = ((true, false), (true, true), (false, false));
    let (map, tab, escape) = (
        (true, false, false),
        (false, true, false),
        (false, false, true),
    );
    assert_eq!(next_state(false, on_foot, map), Some(Computer::Open));
    assert_eq!(next_state(false, on_foot, tab), None);
    assert_eq!(next_state(false, seated, tab), Some(Computer::Open));
    assert_eq!(next_state(false, away, map), None);
    for keys in [map, tab, escape] {
        assert_eq!(next_state(true, on_foot, keys), Some(Computer::Closed));
    }
    assert_eq!(
        next_state(true, away, (false, false, false)),
        Some(Computer::Closed)
    );
    assert_eq!(next_state(true, seated, (false, false, false)), None);
}

/// The map key opens the map and closes it again in the running game.
#[test]
fn the_map_key_opens_and_closes_the_map() {
    let mut app = game("map key");
    let open = |app: &App| *app.world().resource::<State<Computer>>().get() == Computer::Open;
    tap(&mut app, KeyCode::KeyM, None);
    run(&mut app, 2);
    assert!(open(&app));
    tap(&mut app, KeyCode::KeyM, None);
    run(&mut app, 2);
    assert!(!open(&app));
}
