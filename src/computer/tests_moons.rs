//! Computer tests: moons in the list and on the map, with hand-built rows until the universe
//! has moons.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::chart::{Dot, chart, hit};
use super::lens::Lens;
use super::pointer::{MapView, tap};
use super::show::cell_text;
use super::sift::arrange;
use super::view::Part;
use super::*;
use crate::space::UniPos;
use crate::universe::{BodyId, StarId};

/// Returns a row named `title` for `target`, whose centre is `at` metres from the star.
fn row(title: &str, target: Target, at: DVec3) -> Line {
    Line {
        title: title.to_string(),
        detail: String::new(),
        target,
        distance: at.length(),
        visited: true,
        at: UniPos::default() + at,
    }
}

/// Returns the body `index` of the default star, or its moon `moon`.
fn body(index: u8, moon: u8) -> Target {
    Target::Body(BodyId {
        index,
        moon,
        ..default()
    })
}

/// A star, a planet "Rhea" with the moons "Ola" and "Zed", and a planet "Dune" with the moon
/// "Ash", as the system tab builds them: each planet followed by its moons outward.
fn system() -> Vec<Line> {
    let (east, north) = (DVec3::X, DVec3::NEG_Z);
    vec![
        row("Vereya", Target::Star(StarId::default()), DVec3::ZERO),
        row("Rhea", body(0, 0), east * 1.0e10),
        row("Zed", body(0, 1), east * 1.0e10 + north * 4.0e8),
        row("Ola", body(0, 2), east * 1.0e10 - east * 9.0e8),
        row("Dune", body(1, 0), north * 4.0e10),
        row("Ash", body(1, 1), north * 4.0e10 + east * 3.0e8),
    ]
}

/// Returns the names of `lines` in order.
fn names(lines: Vec<Line>) -> Vec<String> {
    lines.into_iter().map(|line| line.title).collect()
}

/// By name each planet comes before its moons, the moons sorted among themselves; by distance
/// the table stays outward. The filter keeps a planet with its moons and a moon with its planet.
#[test]
fn moons_stay_with_their_planet_in_the_list() {
    let near = names(arrange(system(), "", Sort::Near));
    assert_eq!(near, ["Vereya", "Rhea", "Zed", "Ola", "Dune", "Ash"]);
    let by_name = names(arrange(system(), "", Sort::Name));
    assert_eq!(by_name, ["Dune", "Ash", "Rhea", "Ola", "Zed", "Vereya"]);
    assert_eq!(
        names(arrange(system(), "rhe", Sort::Near)),
        ["Rhea", "Zed", "Ola"]
    );
    assert_eq!(names(arrange(system(), "ash", Sort::Name)), ["Dune", "Ash"]);
    assert_eq!(names(arrange(system(), "ola", Sort::Near)), ["Rhea", "Ola"]);
    assert!(arrange(system(), "xx", Sort::Name).is_empty());
    let marks = (false, false);
    let title = |line: &Line| cell_text(line, Part::Title, marks).0;
    assert_eq!(
        (title(&system()[2]), title(&system()[1])),
        ("  Zed".into(), "Rhea".into())
    );
    assert_eq!(cell_text(&system()[2], Part::Detail, marks).0, "");
}

/// Returns the chart point of row `index` among `dots`.
fn point(dots: &[(Dot, Vec2)], index: usize) -> Vec2 {
    dots.iter()
        .find(|(dot, _)| *dot == Dot::Row(index))
        .unwrap()
        .1
}

/// A moon sits one step per rank from its planet, toward where it really lies, and has no
/// orbit around the star. The ship next to a moon is drawn by that moon.
#[test]
fn moons_sit_on_tiny_orbits_around_their_planet() {
    let mut lines = system();
    let ship = lines[2].at + DVec3::X * 2.0e4;
    for line in &mut lines {
        line.distance = line.at.distance(ship);
    }
    let dots = chart(Tab::System, &lines, ship);
    let (rhea, zed, ola) = (point(&dots, 1), point(&dots, 2), point(&dots, 3));
    assert!(
        (zed - (rhea + Vec2::Y * MOON_STEP)).length() < 1e-5,
        "{zed} {rhea}"
    );
    assert!(
        (ola - (rhea - Vec2::X * 2.0 * MOON_STEP)).length() < 1e-5,
        "{ola}"
    );
    let ash = point(&dots, 5) - point(&dots, 4);
    assert!((ash - Vec2::X * MOON_STEP).length() < 1e-5);
    let orbits = dots.iter().filter(|(dot, _)| *dot == Dot::Orbit).count();
    assert_eq!(orbits, 2 * 40);
    let drawn = dots.iter().find(|(dot, _)| *dot == Dot::Ship).unwrap().1;
    assert!(drawn.distance(zed) < 1e-4, "{drawn} {zed}");
    let sorted = arrange(lines.clone(), "", Sort::Name);
    let centred = chart(Tab::System, &sorted, ship);
    assert_eq!(point(&centred, 5), Vec2::ZERO);
}

/// Below the zoom that parts them a moon is hidden and the pointer finds its planet; from
/// that zoom the pointer finds the moon, and two clicks on it choose it like any body.
#[test]
fn a_moon_is_pointed_at_once_the_zoom_parts_it() {
    let lines = system();
    let dots = chart(Tab::System, &lines, UniPos::default());
    let zed = point(&dots, 2);
    let shown = |zoom: f32| {
        let kept = dots.iter().filter(|(dot, _)| seen(*dot, &lines, zoom));
        kept.copied().collect::<Vec<_>>()
    };
    let whole = Lens::default();
    assert_eq!(
        hit(&shown(1.0), whole, whole.to_map(zed), 10.0),
        Some(Dot::Row(1))
    );
    assert_eq!(shown(1.0).len(), dots.len() - 3);
    let close = whole.zoomed(whole.to_map(zed), MOON_ZOOM);
    assert_eq!(
        hit(&shown(MOON_ZOOM), close, close.to_map(zed), 4.0),
        Some(Dot::Row(2))
    );
    let mut board = Board {
        lines: system(),
        ..default()
    };
    let mut view = MapView {
        hover: Some(Dot::Row(2)),
        ..default()
    };
    assert!(!tap(&mut view, &mut board, 1.0) && board.cursor == 2);
    assert!(tap(&mut view, &mut board, 1.2));
}
