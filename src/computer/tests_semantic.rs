//! Computer tests: the wheel moving from one chart to the next at the ends of the zoom, and
//! the lens easing to its goal.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::chart::Dot;
use super::lens::{Lens, MAX_ZOOM};
use super::pointer::{MapView, plot, zoom};
use super::semantic::{DIVE, entry, inner, outer, shift};
use super::tests_pointer::{pixel, point, rig};
use super::*;

/// The wheel changes chart only at the thresholds: out of a chart seen whole, and into the
/// place of the ship from the dive zoom on. Anywhere else it zooms.
#[test]
fn the_wheel_changes_chart_at_the_thresholds() {
    assert_eq!(shift(Tab::System, 1.0, -1.0, false), Some(Tab::Galaxy));
    assert_eq!(shift(Tab::Galaxy, 1.0, -1.0, true), Some(Tab::Universe));
    assert_eq!(shift(Tab::Universe, 1.0, -1.0, false), None);
    assert_eq!(shift(Tab::System, 1.01, -1.0, false), None);
    assert_eq!(shift(Tab::Galaxy, DIVE, 1.0, true), Some(Tab::System));
    assert_eq!(shift(Tab::Universe, MAX_ZOOM, 1.0, true), Some(Tab::Galaxy));
    assert_eq!(shift(Tab::Galaxy, DIVE, 1.0, false), None);
    assert_eq!(shift(Tab::Galaxy, DIVE - 0.01, 1.0, true), None);
    assert_eq!(shift(Tab::System, MAX_ZOOM, 1.0, true), None);
    assert_eq!(shift(Tab::Galaxy, 1.0, 0.0, true), None);
    for tab in Tab::ALL {
        assert!(outer(tab).is_none_or(|out| inner(out) == Some(tab)));
        assert!(inner(tab).is_none_or(|within| outer(within) == Some(tab)));
    }
}

/// A chart reached by zooming out opens close on the ship and pulls back, so one notch in
/// leads back inside. Any other way a chart opens whole.
#[test]
fn a_chart_opens_as_the_way_there_asks() {
    let (start, goal) = entry(Tab::System, Tab::Galaxy);
    assert_eq!((start.zoom, start.centre), (DIVE, Vec2::ZERO));
    assert!(goal.zoom < DIVE && goal.zoom > 1.0 && goal.centre == Vec2::ZERO);
    assert_eq!(entry(Tab::Galaxy, Tab::Universe).0.zoom, DIVE);
    let whole = (Lens::default(), Lens::default());
    assert_eq!(entry(Tab::Galaxy, Tab::System), whole);
    assert_eq!(entry(Tab::System, Tab::Universe), whole);
    assert_eq!(entry(Tab::Universe, Tab::Universe), whole);
}

/// The lens closes a share of the gap to its goal at each step, the zoom by ratio, and lands
/// exactly on the goal instead of creeping toward it for ever.
#[test]
fn the_lens_eases_to_its_goal_and_lands_on_it() {
    let goal = Lens {
        zoom: 16.0,
        centre: Vec2::new(0.5, -0.25),
    };
    let half = Lens::default().toward(goal, 0.5);
    assert!((half.zoom - 4.0).abs() < 1e-4 && half.centre.distance(goal.centre * 0.5) < 1e-5);
    assert_eq!(Lens::default().toward(goal, 1.0), goal);
    assert_eq!(goal.toward(goal, 0.2), goal);
    let mut lens = Lens::default();
    let mut steps = 0;
    while lens != goal && steps < 200 {
        let next = lens.toward(goal, 0.18);
        assert!(next.zoom >= lens.zoom && next.zoom <= goal.zoom);
        (lens, steps) = (next, steps + 1);
    }
    assert!(lens == goal && steps > 10 && steps < 80, "{steps} steps");
}

/// Turns the wheel by `notches` over the map and lays the chart out again, with the same
/// rows when the wheel opened another chart.
fn turn(app: &mut App, notches: f32) -> (Tab, Lens, Lens) {
    app.insert_resource(AccumulatedMouseScroll {
        unit: MouseScrollUnit::Line,
        delta: Vec2::new(0.0, notches),
    });
    let before = app.world().resource::<Board>().clone();
    app.world_mut().run_system_once(zoom).unwrap();
    let mut board = app.world_mut().resource_mut::<Board>();
    if board.stale {
        board.ship = before.ship;
        board.fill("Chart".to_string(), before.lines);
    }
    app.world_mut().run_system_once(plot).unwrap();
    let view = app.world().resource::<MapView>();
    (view.tab, view.lens, view.goal)
}

/// On the map, one notch out of the whole system chart opens the galaxy chart close on the
/// ship, and from there a notch into the place of the ship at the dive zoom opens the system
/// again, while a notch anywhere else only zooms.
#[test]
fn the_wheel_moves_between_the_charts_on_the_map() {
    let mut app = rig();
    point(&mut app, Vec2::new(30.0, 30.0));
    let (tab, lens, goal) = turn(&mut app, -1.0);
    assert_eq!((tab, lens.zoom, goal.zoom), (Tab::Galaxy, DIVE, DIVE * 0.5));
    let (tab, _, goal) = turn(&mut app, 1.0);
    assert!(tab == Tab::Galaxy && goal.zoom > DIVE * 0.5);
    let mut view = app.world_mut().resource_mut::<MapView>();
    let centre = view.dots[0].1;
    let close = Lens { zoom: DIVE, centre }.clamped();
    (view.lens, view.goal) = (close, close);
    let home = app.world().resource::<Board>().lines[0].target;
    app.world_mut().resource_mut::<Board>().here = [None, Some(home), None];
    let star = pixel(&app, Dot::Row(0));
    point(&mut app, star);
    assert_eq!(app.world().resource::<MapView>().hover, Some(Dot::Row(0)));
    let (tab, lens, goal) = turn(&mut app, 1.0);
    assert_eq!(
        (tab, lens, goal),
        (Tab::System, Lens::default(), Lens::default())
    );
}
