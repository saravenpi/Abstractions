//! Computer tests: the charts as nested levels, the wheel that stays on the chart shown, and
//! the lens easing to its goal.

use bevy::ecs::system::RunSystemOnce;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::chart::Dot;
use super::lens::{Lens, MAX_ZOOM};
use super::pointer::{MapView, plot, zoom};
use super::semantic::{DIVE, entry, outer};
use super::tests_pointer::{pixel, point, rig};
use super::*;

/// The charts nest from the system out to the universe, which has nothing around it.
#[test]
fn the_charts_nest_out_to_the_universe() {
    assert_eq!(outer(Tab::System), Some(Tab::Galaxy));
    assert_eq!(outer(Tab::Galaxy), Some(Tab::Universe));
    assert_eq!(outer(Tab::Universe), None);
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
/// rows if the table was asked for again.
pub(super) fn turn(app: &mut App, notches: f32) -> (Tab, Lens, Lens) {
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

/// On the map the wheel only zooms the chart shown, within its own zoom: out of a chart seen
/// whole, or in at the closest zoom on the place of the ship, it stays on the same chart.
#[test]
fn the_wheel_never_changes_the_chart() {
    let mut app = rig();
    point(&mut app, Vec2::new(30.0, 30.0));
    let (tab, lens, goal) = turn(&mut app, -1.0);
    assert_eq!(
        (tab, lens, goal),
        (Tab::System, Lens::default(), Lens::default())
    );
    for _ in 0..40 {
        assert_eq!(turn(&mut app, -3.0).0, Tab::System);
    }
    let home = app.world().resource::<Board>().lines[0].target;
    app.world_mut().resource_mut::<Board>().here = [None, Some(home), None];
    let star = pixel(&app, Dot::Row(0));
    point(&mut app, star);
    for _ in 0..60 {
        let (tab, _, goal) = turn(&mut app, 2.0);
        assert!(tab == Tab::System && goal.zoom <= MAX_ZOOM);
    }
    let view = app.world().resource::<MapView>();
    assert_eq!(view.goal.zoom, MAX_ZOOM);
    assert_eq!(app.world().resource::<Board>().tab, Tab::System);
}
