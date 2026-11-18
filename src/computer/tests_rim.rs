//! Computer tests: the map keeps a pin on its rim toward a home it does not show, and draws
//! a chart to its near rows when home lies far beyond them.

use bevy::math::Vec2;

use super::map::SIZE;
use super::pointer::MapView;
use super::rim::point;
use super::tests_home::{far, stranger};
use super::*;
use crate::universe::Universe;

/// The pin sits on the edge of the map the way home leaves through, and nowhere while the map
/// shows home. A chart whose home lies past a gap is drawn to the rows before the gap.
#[test]
fn the_rim_pin_points_toward_home() {
    let inset = Vec2::new(36.0, 15.0);
    assert_eq!(rim(Vec2::new(120.0, 10.0), 240.0, inset), None);
    let east = rim(Vec2::new(900.0, 120.0), 240.0, inset).unwrap();
    assert_eq!(east, Vec2::new(204.0, 120.0));
    let north_west = rim(Vec2::new(-880.0, -880.0), 240.0, inset).unwrap();
    assert!(north_west.distance(Vec2::splat(36.0)) < 1e-3);
    let near = [(Dot::Row(0), Vec2::X * 0.88), (Dot::Row(1), Vec2::Y * 0.7)];
    let (mut dots, mut span) = (near, Some(1.0e3));
    spread(&mut dots, &mut span, Some(0));
    assert_eq!((dots, span), (near, Some(1.0e3)));
    let mut dots = [(Dot::Row(0), Vec2::X * 0.88), (Dot::Row(1), Vec2::Y * 0.11)];
    spread(&mut dots, &mut span, Some(0));
    assert!((dots[1].1.y - 0.88).abs() < 1e-6 && (dots[0].1.x - 7.04).abs() < 1e-4);
    assert!((span.unwrap() - 125.0).abs() < 1e-3);
    assert_eq!((rank(true, 3), rank(false, 0), rank(false, 3)), (0, 1, 4));
}

/// Returns the universe tab of a ship ten cells east of home that never visited it.
fn far_board(universe: &Universe) -> Board {
    let atlas = stranger(universe);
    let (cell, ship) = far(universe);
    let atlas = &atlas;
    let lines = View {
        universe,
        atlas,
        ship,
        time: 0.0,
    }
    .universe();
    let here = [None, None, Some(Target::Galaxy(cell))];
    let mut board = Board {
        tab: Tab::Universe,
        ship,
        home: homes(universe),
        beacon: beacon(universe, here, 0.0),
        ..Default::default()
    };
    board.fill(
        String::new(),
        arranged(lines, homes(universe)[2], "", Sort::Near),
    );
    board
}

/// Ten cells east of home, the universe chart leaves home off the map and the pin sits on
/// the west edge, at the height of the ship. On a chart that does not list home the pin
/// still points the way.
#[test]
fn the_pin_of_a_far_home_sits_on_the_west_edge() {
    let mut board = far_board(&Universe::new(7));
    let ship = board.ship;
    let mut view = MapView {
        dots: chart(board.tab, &board.lines, ship),
        span: span(board.tab, &board.lines, ship),
        ..Default::default()
    };
    assert_eq!(home_row(&board), Some(0));
    spread(&mut view.dots, &mut view.span, home_row(&board));
    let at = point(&board, &view).unwrap();
    assert!(at.x < -1.0, "{at}");
    let pin = rim(view.lens.to_map(at), SIZE, Vec2::splat(20.0)).unwrap();
    assert!((pin.x - 20.0).abs() < 1e-3 && (pin.y - 0.5 * SIZE).abs() < 20.0);
    (board.tab, board.lines) = (Tab::System, Vec::new());
    (view.dots, view.span) = (Vec::new(), None);
    let off = point(&board, &view).unwrap();
    assert!(off.x < -100.0 && rim(view.lens.to_map(off), SIZE, Vec2::ZERO).unwrap().x < 1.0);
}
