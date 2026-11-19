//! Computer tests: the wheel by lines and by pixels, the pinch, the drag that
//! keeps the grabbed point under the pointer, the drift after it and the pan keys.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::coast::{STOP, keyed};
use super::hand::PIXELS_PER_NOTCH;
use super::map::SIZE;
use super::pointer::zoom;
use super::tests_pointer::{click, point, rig};
use super::*;

/// Returns one frame of scrolling by `delta` in `unit`.
fn scroll(unit: MouseScrollUnit, delta: Vec2) -> AccumulatedMouseScroll {
    AccumulatedMouseScroll { unit, delta }
}

/// A line of the wheel zooms by one notch, a trackpad or smooth wheel zooms too, a hundred
/// pixels as much as a line, and a pinch zooms by the share it grew.
#[test]
fn the_wheel_and_the_trackpad_zoom() {
    assert_eq!(turn(&scroll(MouseScrollUnit::Line, Vec2::Y)), 1.0);
    let swipe = scroll(MouseScrollUnit::Pixel, Vec2::new(12.0, -30.0));
    assert_eq!(turn(&swipe), -30.0 / PIXELS_PER_NOTCH);
    assert_eq!(turn(&scroll(MouseScrollUnit::Pixel, Vec2::ZERO)), 0.0);
    let hand = Hand {
        pinch: 0.15,
        ..default()
    };
    let notches = hand.notches(0.0);
    assert!((ZOOM_STEP.powf(notches) - 1.15).abs() < 1e-4);
}

/// A trackpad scroll over the chart zooms it and never changes the level, however long it lasts.
#[test]
fn a_trackpad_scroll_zooms_the_chart_and_keeps_the_level() {
    let mut app = rig();
    let closer = Lens::default().zoomed(Vec2::splat(SIZE * 0.5), 4.0);
    let mut view = app.world_mut().resource_mut::<MapView>();
    (view.lens, view.goal) = (closer, closer);
    point(&mut app, Vec2::new(60.0, 60.0));
    app.insert_resource(scroll(MouseScrollUnit::Pixel, Vec2::new(0.0, -400.0)));
    for _ in 0..60 {
        app.world_mut().run_system_once(zoom).unwrap();
    }
    let view = app.world().resource::<MapView>();
    assert!(view.goal.zoom < 4.0 && view.tab == Tab::System);
    assert_eq!(app.world().resource::<Board>().tab, Tab::System);
}

/// A drag pans the chart one to one: the chart point grabbed stays under the pointer, and the
/// drag leaves no click behind.
#[test]
fn a_drag_keeps_the_grabbed_point_under_the_pointer() {
    let mut app = rig();
    let closer = Lens::default().zoomed(Vec2::splat(SIZE * 0.5), 6.0);
    let mut view = app.world_mut().resource_mut::<MapView>();
    (view.lens, view.goal, view.dots) = (closer, closer, Vec::new());
    let from = Vec2::new(100.0, 150.0);
    point(&mut app, from);
    let grabbed = closer.to_chart(from);
    let drag = Vec2::new(-55.0, 32.0);
    click(&mut app, drag);
    let view = app.world().resource::<MapView>();
    assert!(view.lens.to_map(grabbed).distance(from + drag) < 1e-2);
    assert!(view.clicked.is_none() && !view.picked);
}

/// A fast drag leaves the map drifting, and the drift dies down to a stop within a second or
/// two of frames. A still pointer at the end of a drag leaves no drift.
#[test]
fn a_drag_drifts_on_and_slows_to_a_stop() {
    let mut hand = Hand::default();
    for _ in 0..10 {
        hand.follow(Vec2::new(10.0, 0.0), 1.0 / 60.0);
    }
    assert!(hand.drift.x > 400.0 && hand.drift.y == 0.0);
    let mut moved = Vec2::ZERO;
    for _ in 0..120 {
        moved += hand.coast(Vec2::ZERO, 1.0 / 60.0);
    }
    assert!(moved.x > 50.0 && moved.x < 200.0 && hand.drift == Vec2::ZERO);
    for _ in 0..30 {
        hand.follow(Vec2::new(10.0, 0.0), 1.0 / 60.0);
    }
    for _ in 0..30 {
        hand.follow(Vec2::ZERO, 1.0 / 60.0);
    }
    assert!(hand.drift.length() < STOP);
}

/// W, A, S and D pan whenever asked; the arrows only while the pointer is over the map. W
/// shows what is north, so it moves the picture down.
#[test]
fn the_pan_keys_move_the_picture() {
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyW);
    assert_eq!(keyed(&keys, false), Vec2::Y);
    keys.press(KeyCode::ArrowLeft);
    assert_eq!(keyed(&keys, false), Vec2::Y);
    assert_eq!(keyed(&keys, true), Vec2::new(1.0, 1.0).normalize());
    let mut hand = Hand::default();
    let step = hand.coast(keyed(&keys, false), 0.5);
    assert!(step.y > 0.0 && step.x == 0.0);
    keys.release_all();
    assert_eq!(keyed(&keys, true), Vec2::ZERO);
}
