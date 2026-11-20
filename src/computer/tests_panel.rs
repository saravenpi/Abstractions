//! Computer tests: the measures of the side panel, the words of the footer, and the open panel
//! leaving the game running.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::course::{arrival, course};
use super::layout::{FACTS, Measures, RESULTS, ROW, SIDE, SMALLEST, fixed, measure};
use super::tests_rig::{game, run, tap};
use super::view::{Panel, Slot};
use super::*;
use crate::menus::Menu;
use crate::screens::{in_control, playing};

/// At every window size the side panel keeps its width, the map is the largest square beside
/// it, and the search results take what the panel leaves within their fewest and most rows.
/// At the smallest layout, 1280 by 720 logical pixels, nothing listed leaves one result beside
/// the whole top of the card, and the card is cut below its first rows: its tallest form, 17
/// rows at the new type scale, needs more than 720 pixels.
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
        map: 720.0,
        rows: RESULTS.0,
        lists: (0, 0),
    };
    assert_eq!(measure(SMALLEST, (0, 0)), smallest);
    let usual = measure(Vec2::new(1280.0, 720.0), (0, 0));
    assert!(fixed((0, 0)) - FACTS + usual.rows as f32 * ROW <= 720.0);
}

/// The course line gives a time to arrival only while the ship moves, and says nothing without
/// a course.
#[test]
fn the_footer_says_the_right_words() {
    assert_eq!(arrival(6000.0, 100.0).as_deref(), Some("about 1 min"));
    assert_eq!(arrival(6000.0, 0.5), None);
    assert_eq!(
        course(Some(("Kolva", 1500.0)), 0.0),
        "Course Kolva - 1.5 km"
    );
    let moving = course(Some(("Kolva", 720_000.0)), 100.0);
    assert_eq!(moving, "Course Kolva - 720 km - about 2 h");
    assert_eq!(course(None, 100.0), "");
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
