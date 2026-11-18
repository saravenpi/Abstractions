//! Computer tests: the measures of the side panel, the words of its header and footer, and
//! the open panel leaving the game running.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::course::{NOWHERE, arrival, course, crumbs, primary, words};
use super::detail::{LINES, readout};
use super::layout::{CONTENT, MIN_ROWS, Measures, ROW, SMALLEST, chrome, measure};
use super::tests_detail::{records, world};
use super::tests_rig::{game, run, tap};
use super::view::{Panel, Slot};
use super::*;
use crate::menus::Menu;
use crate::screens::{in_control, playing};
use crate::ui::theme;
use crate::universe::{GalaxyId, PlanetSpec};
use bevy::math::IVec3;

/// At every window size the sections, the map and the rows fit the height with the same
/// margin around and between them, at least 24 pixels from 1080 pixels up. The map is as wide
/// as the panel inside its margins, unless the card needs the panel wider than the map.
#[test]
fn the_panel_fits_every_window() {
    for height in [540.0, 600.0, 720.0, 900.0, 1080.0, 1200.0, 1440.0, 2160.0] {
        for width in [960.0, 1280.0, 1920.0, 2560.0, 3840.0] {
            let window = Vec2::new(width, height);
            let fit = measure(window);
            let used = chrome(fit.margin) + fit.map + fit.rows as f32 * ROW;
            let inside = fit.panel - 2.0 * fit.margin;
            assert!(used <= height, "{window}: {used}");
            assert!(fit.map >= 120.0 && inside >= CONTENT.0, "{window}: {fit:?}");
            assert!(
                fit.map == inside || inside == CONTENT.0,
                "{window}: {fit:?}"
            );
            assert!((MIN_ROWS..=VISIBLE_ROWS).contains(&fit.rows), "{window}");
            assert!(fit.margin >= 12.0 && (height < 1080.0 || fit.margin >= 24.0));
        }
    }
    let smallest = Measures {
        margin: 12.0,
        panel: 420.0,
        map: 121.0,
        rows: MIN_ROWS,
    };
    assert_eq!(measure(SMALLEST), smallest);
    let full = Measures {
        margin: 24.0,
        panel: 473.0,
        map: 425.0,
        rows: 7,
    };
    assert_eq!(measure(Vec2::new(1920.0, 1080.0)), full);
}

/// Every line of the readout of every class fits the narrowest panel without wrapping.
#[test]
fn the_card_fits_the_narrowest_panel() {
    let glyph = theme::FONT_SMALL * theme::GLYPH_ASPECT;
    let room = (CONTENT.0 / glyph).floor() as usize;
    for record in records() {
        let (line, planet) = world(record);
        let slow = PlanetSpec {
            radius: 12_300.0,
            day_seconds: 3599.0,
            orbit_seconds: 172_799.0,
            ..planet
        };
        let lines = readout(&line, Some(&slow));
        assert_eq!(lines.len(), LINES);
        let longest = lines.iter().map(String::len).max().unwrap();
        assert!(longest <= room, "{lines:?}");
    }
}

/// The breadcrumb joins the names from the galaxy down, the main button offers the next
/// thing to do with the course, and the course line gives a time to arrival only while the
/// ship moves.
#[test]
fn the_header_and_the_footer_say_the_right_words() {
    let names = ["Aru".to_string(), "Vereya".to_string(), "Kodya".to_string()];
    assert_eq!(crumbs(&names), "Aru > Vereya > Kodya");
    assert_eq!(crumbs(&names[..1]), "Aru");
    assert_eq!(crumbs(&[]), NOWHERE);
    let here = Some(Target::Galaxy(GalaxyId::default()));
    let there = Some(Target::Galaxy(GalaxyId(IVec3::X)));
    let (idle, flying) = (RouteStatus::Idle, RouteStatus::InWarp);
    assert_eq!(primary(here, None, idle), Some(Act::Set));
    assert_eq!(primary(here, here, idle), Some(Act::Engage));
    assert_eq!(primary(None, here, idle), Some(Act::Engage));
    assert_eq!(primary(here, here, flying), Some(Act::Clear));
    assert_eq!(primary(there, here, flying), Some(Act::Set));
    assert_eq!(primary(None, None, idle), None);
    assert_eq!(words(Act::Set, "Kodya b"), "Set course to Kodya b");
    assert_eq!(words(Act::Engage, "Kodya b"), "Engage autopilot");
    assert_eq!(words(Act::Clear, ""), "Clear course");
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

/// Opening the panel leaves the game running: physics and the game clock are not paused and
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
    assert_eq!((panel.width, panel.right), (px(420), px(0)));
    let mut slots = app.world_mut().query::<(&Slot, &Node)>();
    let drawn = slots
        .iter(app.world())
        .filter(|(_, node)| node.display != Display::None);
    assert_eq!(drawn.count(), MIN_ROWS);
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
