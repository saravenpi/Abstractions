//! Computer tests: pointing at the map, clicking it and dragging it, in an app without a window.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::ui::RelativeCursorPosition;

use super::chart::Dot;
use super::keys::on_activate;
use super::lens::Lens;
use super::map::{Field, SIZE};
use super::pointer::{DOUBLE_CLICK, MapView, aim, plot, press, tap};
use super::readout::More;
use super::view::Slot;
use super::*;
use crate::audio::AudioCue;
use crate::universe::Universe;

/// Adds the resources the buttons and the pointer read.
fn resources(app: &mut App) {
    app.init_resource::<Destination>()
        .init_resource::<MapView>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<AccumulatedMouseScroll>()
        .init_resource::<Hand>()
        .init_resource::<More>()
        .init_resource::<RouteStatus>();
}

/// Builds an app that holds the open screen of the home system without drawing it: the
/// table, the map field, the row buttons and the chart laid out.
pub(super) fn rig() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<Computer>()
        .add_message::<AudioCue>()
        .add_observer(on_activate);
    resources(&mut app);
    let universe = Universe::new(11);
    let (atlas, star) = (Atlas::new(&universe), universe.home().star);
    let ship = universe.star_position(star).unwrap() + bevy::math::DVec3::X * 3.0e9;
    let (universe, atlas, time) = (&universe, &atlas, 50.0);
    let lines = View {
        universe,
        atlas,
        ship,
        time,
    }
    .system(star);
    let mut board = Board { ship, ..default() };
    board.fill("Home system".to_string(), lines);
    app.insert_resource(board);
    app.world_mut()
        .spawn((Field, RelativeCursorPosition::default()));
    app.world_mut().spawn_batch((0..VISIBLE_ROWS).map(Slot));
    app.world_mut().run_system_once(plot).unwrap();
    app
}

/// Returns the pixel of the map the mark `dot` is drawn at.
pub(super) fn pixel(app: &App, dot: Dot) -> Vec2 {
    let view = app.world().resource::<MapView>();
    let found = view.dots.iter().find(|(other, _)| *other == dot).unwrap();
    view.goal.to_map(found.1)
}

/// Moves the pointer to `at`, a pixel of the map.
pub(super) fn point(app: &mut App, at: Vec2) {
    let share = at / SIZE - 0.5;
    let over = share.abs().max_element() <= 0.5;
    let mut fields = app.world_mut().query::<&mut RelativeCursorPosition>();
    for mut field in fields.iter_mut(app.world_mut()) {
        (field.cursor_over, field.normalized) = (over, Some(share));
    }
    app.world_mut().run_system_once(aim).unwrap();
}

/// Holds the left button down for a frame, moves the pointer by `drag`, then lets go.
pub(super) fn click(app: &mut App, drag: Vec2) {
    let at = app.world().resource::<MapView>().at;
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.press(MouseButton::Left);
    app.world_mut().run_system_once(press).unwrap();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
    if drag != Vec2::ZERO {
        point(app, at + drag);
        app.world_mut().run_system_once(press).unwrap();
    }
    let mut buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
    buttons.release(MouseButton::Left);
    app.world_mut().run_system_once(press).unwrap();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
}

/// Pointing at the mark of a row only lights it: the cursor stays where the keyboard left it
/// while the pointer moves over marks, the ship and empty map.
#[test]
fn pointing_at_a_mark_lights_it_and_selects_nothing() {
    let mut app = rig();
    let state = |app: &App| {
        let hover = app.world().resource::<MapView>().hover;
        (app.world().resource::<Board>().cursor, hover)
    };
    assert_eq!(state(&app), (0, None));
    let planet = pixel(&app, Dot::Row(2));
    point(&mut app, planet + Vec2::new(3.0, -4.0));
    assert_eq!(state(&app), (0, Some(Dot::Row(2))));
    app.world_mut().resource_mut::<Board>().step(1);
    app.world_mut().run_system_once(aim).unwrap();
    assert_eq!(state(&app), (1, Some(Dot::Row(2))));
    let ship = pixel(&app, Dot::Ship);
    point(&mut app, ship);
    assert_eq!(state(&app), (1, Some(Dot::Ship)));
    point(&mut app, Vec2::new(2.0, 2.0));
    assert_eq!(state(&app), (1, None));
    point(&mut app, planet);
    point(&mut app, planet - Vec2::splat(SIZE * 2.0));
    assert_eq!(state(&app), (1, None));
}

/// A click on a mark moves the cursor to its row and picks it, a slow second click on it keeps
/// it, and a click on the ship or on empty map lets go of the selection. No click sets
/// the destination.
#[test]
fn a_click_on_a_mark_selects_its_row_and_never_sets_the_destination() {
    let mut app = rig();
    let state = |app: &App| {
        let chosen = app.world().resource::<Destination>().0;
        let picked = app.world().resource::<MapView>().picked;
        (app.world().resource::<Board>().cursor, picked, chosen)
    };
    let planet = pixel(&app, Dot::Row(2));
    point(&mut app, planet);
    click(&mut app, Vec2::ZERO);
    assert_eq!(state(&app), (2, true, None));
    let slow = |app: &mut App, now: f32| {
        app.world_mut()
            .resource_scope(|world, mut view: Mut<MapView>| {
                tap(&mut view, &mut world.resource_mut::<Board>(), now);
            });
        state(app)
    };
    assert_eq!(slow(&mut app, 10.0), (2, true, None));
    assert_eq!(slow(&mut app, 20.0), (2, true, None));
    for empty in [pixel(&app, Dot::Ship), Vec2::new(2.0, 2.0)] {
        point(&mut app, empty);
        click(&mut app, Vec2::ZERO);
        assert_eq!(state(&app), (2, false, None));
    }
    let star = pixel(&app, Dot::Row(0));
    point(&mut app, star);
    click(&mut app, Vec2::ZERO);
    assert_eq!(state(&app), (0, true, None));
}

/// A drag pans the zoomed chart and is not a click, and two quick clicks on empty map show
/// the whole chart again while two slow ones do not.
#[test]
fn a_drag_pans_and_a_double_click_shows_the_whole_chart() {
    let mut app = rig();
    let closer = Lens::default().zoomed(Vec2::splat(SIZE * 0.5), 4.0);
    let mut view = app.world_mut().resource_mut::<MapView>();
    (view.lens, view.goal, view.dots) = (closer, closer, Vec::new());
    point(&mut app, Vec2::new(40.0, 40.0));
    click(&mut app, Vec2::new(30.0, 0.0));
    let view = app.world().resource::<MapView>().clone();
    let wanted = closer.panned(Vec2::new(30.0, 0.0));
    assert!(view.lens.centre.distance(wanted.centre) < 1e-5 && view.lens.zoom == 4.0);
    assert!(view.lens != closer && view.clicked.is_none() && view.held.is_none());
    assert_eq!(app.world().resource::<Board>().cursor, 0);
    let mut board = Board::default();
    let mut slow = MapView {
        goal: closer,
        ..default()
    };
    tap(&mut slow, &mut board, 1.0);
    tap(&mut slow, &mut board, 1.0 + 2.0 * DOUBLE_CLICK);
    assert_eq!(slow.goal, closer);
    tap(&mut slow, &mut board, 1.0 + 2.5 * DOUBLE_CLICK);
    assert_eq!((slow.goal, slow.clicked), (Lens::default(), None));
    click(&mut app, Vec2::ZERO);
    click(&mut app, Vec2::ZERO);
    assert_eq!(app.world().resource::<MapView>().goal, Lens::default());
}
