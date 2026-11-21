//! Computer tests: hovering lights and never selects, a click selects, a double click
//! goes into the place and sets no course, the keyboard flow, and the arrow of the ship on the
//! charts.

use std::f32::consts::PI;
use std::f64::consts::FRAC_PI_2;

use bevy::ecs::system::RunSystemOnce;
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Activate;

use super::chart::Dot;
use super::hover::{light, look, surface};
use super::pointer::MapView;
use super::tests_pointer::{click, point, rig};
use super::tests_rig::{game, run, tap};
use super::view::{Action, Slot};
use super::*;
use crate::ui::{Look, Selected, theme};

/// The selected row is the white block, a row under the pointer that is not selected is lit
/// faintly, and the others are bare. In the panel, the pointer on a row of the list or on a
/// mark of the map lights that row and leaves the selection on the first row.
#[test]
fn hovering_a_row_or_a_mark_lights_it_and_selects_nothing() {
    assert_eq!(look(true, true), (Look::Active, false));
    assert_eq!(look(false, true), (Look::Rest, true));
    assert_eq!(look(false, false), (Look::Rest, false));
    assert_eq!(surface(Look::Active, false), theme::SELECT);
    assert_eq!(surface(Look::Rest, true), theme::HOVER);
    assert_eq!(surface(Look::Rest, false), Color::NONE);
    let mut app = rig();
    app.world_mut().resource_mut::<MapView>().hover = Some(Dot::Row(1));
    let rows: Vec<Entity> = (0..3)
        .map(|slot| {
            let over = RelativeCursorPosition {
                cursor_over: slot == 2,
                normalized: None,
            };
            let parts = (Look::default(), Selected(false), BackgroundColor::default());
            app.world_mut().spawn((Slot(slot), over, parts)).id()
        })
        .collect();
    app.world_mut().run_system_once(light).unwrap();
    let lit = |entity: Entity| app.world().get::<Selected>(entity).unwrap().0;
    let shown = app.world().get::<Look>(rows[0]).copied();
    assert_eq!(shown, Some(Look::Active));
    assert!(!lit(rows[0]) && lit(rows[1]) && lit(rows[2]));
    assert_eq!(app.world().resource::<Board>().cursor, 0);
}

/// Returns the last two rows of `board` that are not moons, which the map hides until it is zoomed in.
fn last_planets(board: &Board) -> (usize, usize) {
    let planets: Vec<usize> = (0..board.lines.len())
        .filter(|row| !is_moon(board.lines[*row].target))
        .collect();
    (planets[planets.len() - 1], planets[planets.len() - 2])
}

/// A click on a row of the list selects it and sets no course. Two quick clicks on the mark of a planet
/// that was not selected, since the map hides moons until it is zoomed in, select it, ease the map onto
/// it and ask to go into its place, still with no course.
#[test]
fn a_click_on_a_row_selects_and_a_double_click_on_a_mark_goes_into_it() {
    let mut app = rig();
    let slot = |app: &mut App, wanted: usize| {
        let mut slots = app.world_mut().query::<(Entity, &Slot)>();
        let found = slots.iter(app.world()).find(|(_, slot)| slot.0 == wanted);
        found.unwrap().0
    };
    let (pick, mark) = last_planets(app.world().resource::<Board>());
    let entity = slot(&mut app, pick);
    app.world_mut().trigger(Activate { entity });
    let state = |app: &App| {
        let course = app.world().resource::<Destination>().0;
        (app.world().resource::<Board>().cursor, course)
    };
    assert_eq!(state(&app), (pick, None));
    assert!(app.world().resource::<MapView>().picked);
    let view = app.world().resource::<MapView>();
    let found = view.dots.iter().find(|(dot, _)| *dot == Dot::Row(mark));
    let at = view.lens.to_map(found.unwrap().1);
    point(&mut app, at);
    click(&mut app, Vec2::ZERO);
    click(&mut app, Vec2::ZERO);
    assert_eq!(state(&app), (mark, None));
    let (view, board) = (
        app.world().resource::<MapView>(),
        app.world().resource::<Board>(),
    );
    assert!(view.picked && Some(view.goal) == aimed(board, view, Dot::Row(mark)));
    assert_eq!(view.into, Some(board.lines[mark].target));
}

/// Returns the words of the progress line under the map.
fn said(app: &mut App) -> String {
    let mut lines = app.world_mut().query_filtered::<&Text, With<Action>>();
    let words = lines.iter(app.world()).next();
    words.map(|text| text.0.clone()).unwrap_or_default()
}

/// The keyboard selects and goes: the arrows move the selection, and Enter sends the ship to
/// the row under the cursor, sets the course there and closes the map. The progress line under
/// the map says nothing until the route is flown, and Delete does not clear the course.
#[test]
fn the_keyboard_selects_and_goes() {
    let mut app = game("keys");
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    let at = super::tests_close::elsewhere(&app);
    for _ in 0..at {
        tap(&mut app, KeyCode::ArrowDown, None);
    }
    let row = app.world().resource::<Board>().lines[at].clone();
    assert_eq!(app.world().resource::<Board>().cursor, at);
    assert_eq!(said(&mut app), "");
    tap(&mut app, KeyCode::Enter, None);
    run(&mut app, 3);
    assert_eq!(app.world().resource::<Destination>().0, Some(row.target));
    let state = app.world().resource::<State<Computer>>();
    assert_eq!(*state.get(), Computer::Closed);
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 3);
    tap(&mut app, KeyCode::Delete, None);
    assert_eq!(app.world().resource::<Destination>().0, Some(row.target));
}

/// Returns where an arrow turned clockwise by `angle` from up points on the screen, whose y
/// axis points down.
fn on_screen(angle: f32) -> Vec2 {
    Vec2::new(angle.sin(), -angle.cos())
}

/// A ship pointing along +X on the system plane draws an arrow pointing right, along -Z up,
/// along +Z down and along -X left. A pitched or rolled ship points where its nose heads, and
/// a nose straight up points the way the top faced.
#[test]
fn the_arrow_points_where_the_nose_points_on_the_chart() {
    let point = |nose: DVec3, top: DVec3| on_screen(bearing(nose, top));
    let cases = [
        (DVec3::X, Vec2::X),
        (DVec3::NEG_Z, Vec2::NEG_Y),
        (DVec3::Z, Vec2::Y),
        (DVec3::NEG_X, Vec2::NEG_X),
    ];
    for (nose, wanted) in cases {
        assert!(point(nose, DVec3::Y).distance(wanted) < 1e-5, "{nose}");
    }
    let pitched = DQuat::from_rotation_x(0.5);
    let nose = pitched * DVec3::NEG_Z;
    assert!(nose.y > 0.4 && point(nose, pitched * DVec3::Y).distance(Vec2::NEG_Y) < 1e-5);
    let east = DQuat::from_rotation_y(-FRAC_PI_2) * DQuat::from_rotation_z(0.7);
    assert!(point(east * DVec3::NEG_Z, east * DVec3::Y).distance(Vec2::X) < 1e-5);
    let up = DQuat::from_rotation_x(FRAC_PI_2);
    assert!(point(up * DVec3::NEG_Z, up * DVec3::Y).distance(Vec2::NEG_Y) < 1e-5);
    assert!((bearing(DVec3::Z, DVec3::Y).abs() - PI).abs() < 1e-5);
}
