//! Computer tests: the measures of the side panel, the words of its header and footer, and
//! the open panel leaving the game running.

use avian3d::prelude::{Physics, PhysicsTime};
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use super::course::{NOWHERE, arrival, course, crumbs, offered};
use super::detail::{LINES, readout};
use super::layout::{CHROME, MIN_ROWS, PAD, PANEL_WIDTH, SMALLEST};
use super::layout::{map_side, panel_width, rows_that_fit};
use super::tests_detail::{records, world};
use super::tests_rig::{game, run, tap};
use super::view::{Panel, Slot};
use super::*;
use crate::menus::Menu;
use crate::screens::{in_control, playing};
use crate::ui::theme;
use crate::universe::{GalaxyId, PlanetSpec};

/// The panel takes four tenths of the window between its narrowest and its widest, and at
/// every window size the map and the rows of the list fit the height that is left.
#[test]
fn the_panel_fits_every_window() {
    let widths = [
        (960.0, 420.0),
        (1280.0, 512.0),
        (1920.0, 640.0),
        (3840.0, 640.0),
    ];
    for (window, panel) in widths {
        assert_eq!(panel_width(window), panel);
    }
    for height in [540.0, 600.0, 720.0, 900.0, 1080.0, 1440.0, 2160.0] {
        for width in [960.0, 1280.0, 1920.0, 3840.0] {
            let window = Vec2::new(width, height);
            let (side, rows) = (map_side(window), rows_that_fit(window));
            let used = CHROME + side + rows as f32 * theme::ROW_COMPACT;
            assert!(used <= height, "{window}: {used}");
            assert!(
                side >= 120.0 && side <= panel_width(width) - 2.0 * PAD,
                "{window}"
            );
            assert!((MIN_ROWS..=VISIBLE_ROWS).contains(&rows), "{window}");
        }
    }
    assert_eq!(rows_that_fit(SMALLEST), MIN_ROWS);
    assert_eq!(rows_that_fit(Vec2::new(1920.0, 1080.0)), VISIBLE_ROWS);
}

/// Every line of the readout of every class fits the narrowest panel without wrapping.
#[test]
fn the_card_fits_the_narrowest_panel() {
    let glyph = theme::FONT_SMALL * theme::GLYPH_ASPECT;
    let room = ((PANEL_WIDTH.0 - 2.0 * PAD) / glyph).floor() as usize;
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

/// The breadcrumb joins the names from the galaxy down, the footer offers one main action at
/// a time, and the course line gives a time to arrival only while the ship moves.
#[test]
fn the_header_and_the_footer_say_the_right_words() {
    let names = ["Aru".to_string(), "Vereya".to_string(), "Kodya".to_string()];
    assert_eq!(crumbs(&names), "Aru > Vereya > Kodya");
    assert_eq!(crumbs(&names[..1]), "Aru");
    assert_eq!(crumbs(&[]), NOWHERE);
    let (here, there) = (Target::Galaxy(GalaxyId::default()), None);
    let shown =
        |cursor, set| [Act::Set, Act::Engage, Act::Clear].map(|act| offered(act, cursor, set));
    assert_eq!(shown(Some(here), there), [true, false, false]);
    assert_eq!(shown(Some(here), Some(here)), [false, true, true]);
    assert_eq!(shown(None, Some(here)), [false, true, true]);
    assert_eq!(shown(None, None), [false, false, false]);
    assert_eq!(arrival(6000.0, 100.0).as_deref(), Some("about 1 min"));
    assert_eq!(arrival(6000.0, 0.5), None);
    let idle = RouteStatus::Idle;
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
