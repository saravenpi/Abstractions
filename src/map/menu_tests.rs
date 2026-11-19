//! Map tests: what the menu of a right click offers for each thing under the pointer, where it
//! opens in the smallest window, what a right click finds under the pointer, and the waypoint
//! it sets.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::spot::{on_chart, on_planet};
use super::{Choice, PAD, ROW, Under, WIDTH, choices, place, words};
use crate::computer::{Board, Dot, MapView, Target};
use crate::map::geo::to_chart;
use crate::map::pins::{Pin, Pins};
use crate::map::surface::Surface;
use crate::map::waypoint::{Waypoints, place as set, toggle};
use crate::menus::{Marker, MarkerKind, MarkerPlace, Markers};
use crate::universe::{BodyId, StarId};

/// One of each thing that can be under the pointer.
fn every() -> [Under; 8] {
    let (body, dir) = (BodyId::default(), DVec3::X);
    [
        Under::Ground(body, dir),
        Under::Chart(Vec2::ZERO),
        Under::Place(Target::Body(body)),
        Under::Place(Target::Star(StarId::default())),
        Under::Place(Target::Galaxy(StarId::default().galaxy)),
        Under::Mine(body, dir),
        Under::Favourite(1),
        Under::Player(0, body),
    ]
}

/// Ground takes a waypoint, a favourite, a copy of its coordinates or the centre of the map; a
/// planet can be the course, be focused, take a waypoint or a favourite; a star or a galaxy only
/// the course and the focus; the own waypoint can also be cleared; a favourite renamed or
/// deleted; a player focused or followed. Each line of a menu says something different.
#[test]
fn the_menu_offers_what_suits_what_is_under_the_pointer() {
    use Choice::*;
    let [ground, chart, planet, star, galaxy, mine, favourite, player] = every();
    assert_eq!(choices(&ground), [Waypoint, Favourite, Copy, Centre]);
    assert_eq!(choices(&chart), [Centre]);
    assert_eq!(choices(&planet), [Course, Focus, Waypoint, Favourite]);
    assert_eq!(choices(&star), [Course, Focus]);
    assert_eq!(choices(&galaxy), [Course, Focus]);
    assert_eq!(choices(&mine), [Course, Focus, Unwaypoint, Favourite]);
    assert_eq!(choices(&favourite), [Course, Focus, Rename, Delete]);
    assert_eq!(choices(&player), [Focus, Course]);
    assert_eq!(words(Waypoint, &ground).1, "Set waypoint here");
    assert_eq!(words(Waypoint, &planet).1, "Set waypoint");
    for under in every() {
        let said: Vec<&str> = choices(&under)
            .iter()
            .map(|c| words(*c, &under).1)
            .collect();
        assert!(
            said.iter()
                .enumerate()
                .all(|(at, line)| !said[..at].contains(line))
        );
    }
}

/// The menu opens at the pointer and moves just enough to stay inside a 960 by 540 window,
/// wherever the pointer is.
#[test]
fn the_menu_stays_inside_the_smallest_window() {
    let window = Vec2::new(960.0, 540.0);
    let corners = [0.0, 1.0].map(|x| [0.0, 1.0].map(|y| window * Vec2::new(x, y)));
    for under in every() {
        let lines = choices(&under).len();
        let size = Vec2::new(WIDTH, lines as f32 * ROW + 2.0 * PAD);
        for at in corners.iter().flatten().chain([&(window * 0.5)]) {
            let corner = place(*at, window, lines);
            assert!(corner.min_element() >= 0.0 && (corner + size).cmple(window).all());
        }
    }
    let inside = Vec2::new(100.0, 120.0);
    assert_eq!(place(inside, window, 4), inside);
}

/// Returns a marker of `kind` that belongs to `owner`, on `body` in the direction `dir`.
fn marker(kind: MarkerKind, owner: u32, (body, dir): (BodyId, DVec3)) -> Marker {
    Marker {
        kind,
        place: MarkerPlace::Body(body, dir * 1000.0),
        color: Color::WHITE,
        label: String::new(),
        owner,
    }
}

/// Five markers on `body` and their pins after the pin of the player: a favourite, a friend,
/// a favourite, the own waypoint and the waypoint of a friend.
fn crowd(body: BodyId) -> (Pins, Markers) {
    let dirs = [
        DVec3::X,
        DVec3::Z,
        DVec3::NEG_X,
        DVec3::NEG_Z,
        DVec3::new(1.0, 0.5, 1.0),
    ];
    let kinds = [
        (MarkerKind::Favourite, 0),
        (MarkerKind::Friend, 5),
        (MarkerKind::Favourite, 0),
        (MarkerKind::Waypoint, 0),
        (MarkerKind::Waypoint, 7),
    ];
    let made = (0..5).map(|at| marker(kinds[at].0, kinds[at].1, (body, dirs[at].normalize())));
    let mut pins = vec![Pin {
        at: Vec2::new(0.1, 0.2),
        marker: None,
    }];
    pins.extend((0..5).map(|at| Pin {
        at: to_chart(dirs[at].normalize()),
        marker: Some(at),
    }));
    (Pins(pins), Markers(made.collect()))
}

/// On a planet a right click finds the own waypoint, a favourite or a player by their place in
/// their lists, and the ground under any other marker, under the player or under no marker.
#[test]
fn a_right_click_finds_what_is_under_the_pointer() {
    let body = BodyId::default();
    let (pins, markers) = crowd(body);
    let under = |hover: Option<usize>| {
        let surface = Surface {
            hover,
            point: Some(DVec3::Y),
            ..default()
        };
        on_planet(body, &surface, &pins, &markers)
    };
    assert_eq!(under(None), Some(Under::Ground(body, DVec3::Y)));
    assert!(matches!(under(Some(0)), Some(Under::Ground(..))));
    assert_eq!(under(Some(2)), Some(Under::Player(0, body)));
    assert_eq!(under(Some(3)), Some(Under::Favourite(1)));
    let Some(Under::Mine(on, dir)) = under(Some(4)) else {
        panic!("the own waypoint is not found");
    };
    assert!(on == body && dir.distance(DVec3::NEG_Z) < 1e-4);
    assert!(matches!(under(Some(5)), Some(Under::Ground(..))));
}

/// On a chart, off any mark, a right click finds empty chart at the point under the pointer.
#[test]
fn a_right_click_off_the_marks_finds_empty_chart() {
    let view = MapView {
        hover: Some(Dot::Ship),
        at: Vec2::new(120.0, 120.0),
        ..default()
    };
    assert_eq!(on_chart(&view, &Board::default()), Under::Chart(Vec2::ZERO));
}

/// Setting the waypoint from the menu replaces the one there was, and the waypoint key clears
/// it again.
#[test]
fn the_menu_sets_the_waypoint_in_place_of_the_old_one() {
    let mut waypoints = Waypoints::default();
    let body = BodyId::default();
    assert!(set(&mut waypoints, body, DVec3::X * 3.0).is_some());
    assert_eq!(waypoints.mine.as_ref().map(|mine| mine.dir), Some(DVec3::X));
    let change = set(&mut waypoints, body, DVec3::Y).unwrap();
    assert_eq!(change.0.map(|mine| mine.dir), Some(DVec3::Y));
    assert!(set(&mut waypoints, body, DVec3::ZERO).is_none());
    assert_eq!(toggle(&mut waypoints, None).unwrap().0, None);
    assert!(waypoints.mine.is_none());
}
