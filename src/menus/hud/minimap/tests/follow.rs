//! The ship on the system chart moves with the ship: a small move gives a small move, nothing
//! jumps when the chart is looked at again, and a rebase of the anchor or a change of frame
//! leaves the place where it was.

use bevy::math::{DQuat, DVec3, Vec3};

use crate::computer::{Atlas, Dot, View, ship_dot, ship_position};
use crate::menus::hud::minimap::follow::Charted;
use crate::menus::hud::minimap::orbit::{Scene, scene, signature};
use crate::space::{Anchor, Frame, UniPos};
use crate::universe::Universe;

/// Returns the universe position of a ship `above` metres from the home planet, along +X.
fn ship_at(universe: &Universe, above: f64) -> UniPos {
    let home = universe.home();
    let at = universe.planet_position(home, 0.0) + DVec3::X * above;
    ship_position(universe, Frame::System(home.star), at, 0.0).unwrap()
}

/// Returns the chart of the home system for a ship at `ship`.
fn charted(universe: &Universe, ship: UniPos) -> Scene {
    let atlas = Atlas::new(universe);
    let view = View {
        universe,
        atlas: &atlas,
        ship,
        time: 0.0,
    };
    scene(&view, universe.home().star, None)
}

/// Moving the ship a few metres moves its dot a little, along the whole way and over a long
/// leg, with the chart of one look: no step is larger than the move itself seen on the chart.
#[test]
fn the_ship_dot_moves_in_small_steps() {
    let universe = Universe::new(7);
    let start = ship_at(&universe, 5.0e4);
    let chart = charted(&universe, start);
    let mut last = ship_dot(&chart.lines, &chart.dots, start);
    let mut worst = 0.0_f32;
    for step in 1..=400 {
        let now = ship_at(&universe, 5.0e4 + f64::from(step) * 500.0);
        let at = ship_dot(&chart.lines, &chart.dots, now);
        worst = worst.max(at.distance(last));
        last = at;
    }
    assert!(worst < 0.01, "{worst}");
}

/// The dot a look at the system gives is the dot the frames before it gave: the chart drawn
/// again does not move the ship.
#[test]
fn looking_again_does_not_move_the_ship() {
    let universe = Universe::new(7);
    let before = ship_at(&universe, 5.0e4);
    let old = charted(&universe, before);
    let now = ship_at(&universe, 5.0e4 + 3000.0);
    let fresh = charted(&universe, now);
    let from_old = ship_dot(&old.lines, &old.dots, now);
    let seen = fresh.place(Dot::Ship).unwrap();
    assert!(from_old.distance(seen) < 1e-3, "{from_old} {seen}");
    let own = ship_dot(&fresh.lines, &fresh.dots, now);
    assert!(own.distance(seen) < 1e-4, "{own} {seen}");
}

/// The ship is not part of what makes the chart be drawn again, so a flying ship does not
/// make it be drawn again every look.
#[test]
fn the_ship_does_not_redraw_the_chart() {
    let universe = Universe::new(7);
    let near = charted(&universe, ship_at(&universe, 5.0e4));
    let moved = charted(&universe, ship_at(&universe, 5.0e4 + 800.0));
    assert_eq!(signature(&near), signature(&moved));
}

/// Moving the render origin of the anchor changes how a point renders, not where it is in the
/// frame: the place the minimap reads is the same before and after a rebase.
#[test]
fn a_rebase_leaves_the_place_where_it_was() {
    let frame = Frame::System(Universe::new(7).home().star);
    let first = Anchor {
        frame,
        origin: DVec3::new(1.0e9, 2.0e3, -5.0e8),
        rotation: DQuat::IDENTITY,
        flat: false,
    };
    let at = first.origin + DVec3::new(120.5, 3.25, -40.0);
    let render = first.to_render(at);
    let later = Anchor {
        origin: first.origin + DVec3::new(100.0, 0.0, -50.0),
        ..first
    };
    let moved = later.to_render(at);
    assert!(render.distance(Vec3::new(120.5, 3.25, -40.0)) < 1e-3);
    assert!(moved.distance(render) > 50.0);
    assert!(later.to_frame(moved).distance(at) < 1e-3);
    assert!(first.to_frame(render).distance(at) < 1e-3);
}

/// The chart goes on between the stars, with the star the ship left, and stops on a planet.
#[test]
fn the_chart_goes_on_between_the_stars() {
    let universe = Universe::new(7);
    let home = universe.home();
    let galaxy = Frame::Galaxy(home.star.galaxy);
    let mut book = Charted::default();
    assert_eq!(book.star_in(galaxy), None);
    assert_eq!(book.star_in(Frame::System(home.star)), Some(home.star));
    assert_eq!(book.star_in(galaxy), Some(home.star));
    assert_eq!(book.star_in(Frame::Surface(home)), None);
    assert_eq!(book.star_in(galaxy), None);
}
