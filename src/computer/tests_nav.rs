//! Computer tests: hovering lights and never selects, a click selects, a second click or a
//! double click sets the course, the keyboard flow, and the arrow of the ship on the charts.

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
use super::primary::Face;
use super::tests_pointer::{click, pixel, point, rig};
use super::tests_rig::{game, run, tap};
use super::view::Slot;
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
    assert_eq!(surface(Look::Active, false), theme::BLOCK);
    assert_eq!(surface(Look::Rest, true), theme::SELECTED);
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

/// A click on a row of the list selects it and sets no course, and two quick clicks on a mark
/// that was not selected set the course to it at once.
#[test]
fn a_click_on_a_row_selects_and_a_double_click_on_a_mark_sets_the_course() {
    let mut app = rig();
    let slot = |app: &mut App, wanted: usize| {
        let mut slots = app.world_mut().query::<(Entity, &Slot)>();
        let found = slots.iter(app.world()).find(|(_, slot)| slot.0 == wanted);
        found.unwrap().0
    };
    let entity = slot(&mut app, 3);
    app.world_mut().trigger(Activate { entity });
    let state = |app: &App| {
        let course = app.world().resource::<Destination>().0;
        (app.world().resource::<Board>().cursor, course)
    };
    assert_eq!(state(&app), (3, None));
    let mark = pixel(&app, Dot::Row(2));
    point(&mut app, mark);
    click(&mut app, Vec2::ZERO);
    click(&mut app, Vec2::ZERO);
    let target = app.world().resource::<Board>().lines[2].target;
    assert_eq!(state(&app), (2, Some(target)));
}

/// Returns the words the main button says.
fn said(app: &mut App) -> String {
    let mut faces = app.world_mut().query::<(&Face, &Text)>();
    let words = faces
        .iter(app.world())
        .find(|(face, _)| **face == Face::Words);
    words.map(|(_, text)| text.0.clone()).unwrap_or_default()
}

/// The keyboard works as before: the arrows move the selection, Enter sets the course to it,
/// and Delete clears it. The main button always says what it will do next.
#[test]
fn the_keyboard_selects_sets_and_clears_the_course() {
    let mut app = game("keys");
    tap(&mut app, KeyCode::Tab, None);
    run(&mut app, 5);
    tap(&mut app, KeyCode::ArrowDown, None);
    let row = app.world().resource::<Board>().lines[1].clone();
    assert_eq!(app.world().resource::<Board>().cursor, 1);
    assert_eq!(said(&mut app), format!("Set course to {}", row.title));
    tap(&mut app, KeyCode::Enter, None);
    run(&mut app, 2);
    assert_eq!(app.world().resource::<Destination>().0, Some(row.target));
    assert_eq!(said(&mut app), "Engage autopilot");
    tap(&mut app, KeyCode::ArrowUp, None);
    assert_eq!(app.world().resource::<Board>().cursor, 0);
    assert_eq!(app.world().resource::<Destination>().0, Some(row.target));
    tap(&mut app, KeyCode::Delete, None);
    assert_eq!(app.world().resource::<Destination>().0, None);
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
