//! Computer tests: the wheel on the table and on the map, the reset key, and the map
//! following the tab and the table cursor.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::chart::Dot;
use super::keys::navigate;
use super::lens::Lens;
use super::map::SIZE;
use super::pointer::{MapView, RESET_KEY, ZOOM_STEP, plot, zoom};
use super::tests_pointer::{pixel, point, rig};
use super::*;

/// Turns the wheel by `notches` for one frame of the zoom and of the table.
fn turn(app: &mut App, notches: f32) {
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, notches),
    });
    app.world_mut().run_system_once(zoom).unwrap();
    app.world_mut().run_system_once(navigate).unwrap();
    app.init_resource::<AccumulatedMouseScroll>();
}

/// The wheel scrolls the table while the pointer is anywhere but on the map, and zooms the
/// map around the pointer while it is on the map, leaving the table where it is.
#[test]
fn the_wheel_scrolls_the_table_or_zooms_the_map() {
    let mut app = rig();
    let mut board = app.world_mut().resource_mut::<Board>();
    let long: Vec<Line> = board.lines.iter().cycle().take(100).cloned().collect();
    board.fill("Long table".to_string(), long);
    app.world_mut().resource_mut::<MapView>().dots.clear();
    let state = |app: &App| {
        let lens = app.world().resource::<MapView>().lens;
        (app.world().resource::<Board>().top, lens)
    };
    turn(&mut app, -1.0);
    assert_eq!(state(&app), (3, Lens::default()));
    point(&mut app, Vec2::new(-200.0, 100.0));
    turn(&mut app, -1.0);
    assert_eq!(state(&app), (6, Lens::default()));
    let at = Vec2::new(200.0, 60.0);
    point(&mut app, at);
    let held = Lens::default().to_chart(at);
    turn(&mut app, 2.0);
    let (top, lens) = state(&app);
    assert!(top == 6 && (lens.zoom - ZOOM_STEP * ZOOM_STEP).abs() < 1e-4);
    assert!(lens.to_map(held).distance(at) < 1e-3);
    turn(&mut app, -9.0);
    assert_eq!(state(&app), (6, Lens::default()));
}

/// The reset key shows the whole chart again, another tab starts from the whole chart, and
/// a cursor moved with the keyboard to a row off the map brings that row into view.
#[test]
fn the_map_follows_the_reset_key_the_tab_and_the_cursor() {
    let mut app = rig();
    let star = pixel(&app, Dot::Row(0));
    let closer = Lens::default().zoomed(star, 16.0);
    let look = |app: &mut App, lens: Lens| app.world_mut().resource_mut::<MapView>().lens = lens;
    look(&mut app, closer);
    let last = app.world().resource::<Board>().lines.len() - 1;
    let off = pixel(&app, Dot::Row(last));
    assert!(off.min_element() < 0.0 || off.max_element() > SIZE, "{off}");
    app.world_mut().resource_mut::<Board>().step(last as isize);
    app.world_mut().run_system_once(plot).unwrap();
    let shown = pixel(&app, Dot::Row(last));
    assert!(
        shown.min_element() >= 0.0 && shown.max_element() <= SIZE,
        "{shown}"
    );
    assert_eq!(app.world().resource::<MapView>().lens.zoom, closer.zoom);
    let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    keys.press(RESET_KEY);
    app.world_mut().run_system_once(zoom).unwrap();
    assert_eq!(app.world().resource::<MapView>().lens, Lens::default());
    look(&mut app, closer);
    let lines = app.world().resource::<Board>().lines.clone();
    let mut board = app.world_mut().resource_mut::<Board>();
    board.show(Tab::Galaxy);
    board.fill("Galaxy".to_string(), lines);
    app.world_mut().run_system_once(plot).unwrap();
    let view = app.world().resource::<MapView>();
    assert_eq!((view.lens, view.tab), (Lens::default(), Tab::Galaxy));
}
