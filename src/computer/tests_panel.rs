//! Computer tests: the measures of the side panel, the words of its header and footer, and
//! the open panel leaving the game running.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::course::{NOWHERE, arrival, course, crumbs};
use super::layout::{Measures, RESULTS, ROW, SIDE, SMALLEST, fixed, measure};
use super::tests_rig::{game, run, tap};
use super::view::{Panel, Slot};
use super::*;
use crate::menus::Menu;
use crate::screens::{in_control, playing};

/// At every window size the side panel keeps its width, the map is the largest square beside
/// it, and the search results take what the panel leaves within their fewest and most rows.
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
        rows: RESULTS.1,
    };
    assert_eq!(measure(SMALLEST, (0, 0)), smallest);
    for lists in [(0, 0), (2, 1), (5, 0), (3, 3), (1, 6)] {
        let fit = measure(SMALLEST, lists);
        let used = fixed(lists) + fit.rows as f32 * ROW;
        assert!(used <= SMALLEST.y, "{lists:?}: {used}");
    }
}

/// The breadcrumb joins the names from the galaxy down, and the course line gives a time to
/// arrival only while the ship moves.
#[test]
fn the_header_and_the_footer_say_the_right_words() {
    let names = ["Aru".to_string(), "Vereya".to_string(), "Kodya".to_string()];
    assert_eq!(crumbs(&names), "Aru > Vereya > Kodya");
    assert_eq!(crumbs(&names[..1]), "Aru");
    assert_eq!(crumbs(&[]), NOWHERE);
    let idle = RouteStatus::Idle;
    assert_eq!(arrival(6000.0, 100.0).as_deref(), Some("about 1 min"));
    assert_eq!(arrival(6000.0, 0.5), None);
    assert_eq!(
        course(Some(("Kodya", 1500.0)), 0.0, idle),
        "Course Kodya - 1.50 km - idle"
    );
    let moving = course(Some(("Kodya", 720_000.0)), 100.0, idle);
    assert_eq!(moving, "Course Kodya - 720 km - about 2 h 0 min - idle");
    assert!(course(None, 100.0, idle).starts_with("No course set"));
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
