//! Map tests: the waypoint key sets and clears, favourites are named and renamed, a level
//! fades in, the flat map opens where it was asked to look and no closer than its cap, and
//! crowded names on it are left out.

use bevy::input::keyboard::Key;
use bevy::math::{DVec3, Vec2};

use super::fade::{FADE, shade};
use super::favourite::{Favourites, edit};
use super::level::{CLOSEST, GROUND, REFERENCE_MAP, WHOLE, first, framed};
use super::raster::grid::COLS;
use super::{Waypoints, to_chart, toggle};
use crate::universe::BodyId;

#[test]
fn the_waypoint_key_sets_then_clears() {
    let mut waypoints = Waypoints::default();
    let body = BodyId::default();
    assert_eq!(toggle(&mut waypoints, None), None);
    let set = toggle(&mut waypoints, Some((body, DVec3::Y * 500.0))).unwrap();
    let mine = waypoints.mine.clone().unwrap();
    assert_eq!(set.0, Some(mine.clone()));
    assert!(mine.dir.distance(DVec3::Y) < 1e-12 && mine.body == body);
    let cleared = toggle(&mut waypoints, Some((body, DVec3::X))).unwrap();
    assert_eq!((cleared.0, waypoints.mine), (None, None));
}

#[test]
fn a_favourite_is_named_and_renamed() {
    let mut favourites = Favourites::default();
    let index = favourites.save(BodyId::default(), DVec3::X * 3.0);
    assert_eq!((index, favourites.0[0].name.as_str()), (0, "Place 1"));
    let mut name = String::from("Camp");
    assert!(!edit(&mut name, &Key::Character("s".into()), "Place 1"));
    assert!(!edit(&mut name, &Key::Backspace, "Place 1"));
    assert!(edit(&mut name, &Key::Enter, "Place 1"));
    favourites.rename(0, &name);
    assert_eq!(favourites.0[0].name, "Camp");
    favourites.rename(0, "   ");
    assert_eq!(favourites.0[0].name, "Camp");
    assert!(edit(&mut name, &Key::Escape, "Place 1"));
    assert_eq!(name, "Place 1");
}

#[test]
fn a_level_fades_in_and_opens_where_asked() {
    assert_eq!(shade(0.0), 1.0);
    assert!((shade(FADE * 0.5) - 0.5).abs() < 1e-6);
    assert_eq!(shade(FADE * 2.0), 0.0);
    let whole = first(None, None);
    assert_eq!((whole.zoom, whole.centre), (WHOLE, Vec2::ZERO));
    let here = DVec3::new(0.3, 0.2, -0.9);
    let close = first(None, Some(here));
    assert_eq!((close.zoom, GROUND), (CLOSEST, CLOSEST));
    assert!(close.centre.distance(to_chart(here)) < 1e-6);
    let asked = first(Some(DVec3::Z), Some(here));
    assert!(asked.centre.distance(to_chart(DVec3::Z)) < 1e-6);
}

/// At the closest zoom one cell of the planet picture takes at most 6 pixels of the map of a
/// 1280 by 720 window, and the closest zoom still comes closer than the whole planet.
#[test]
fn the_closest_zoom_keeps_cells_small() {
    let cell = CLOSEST * REFERENCE_MAP / COLS as f32;
    assert!(cell <= 6.0 + 1e-4 && cell > 5.0, "{cell}");
    const { assert!(CLOSEST > WHOLE) };
}

/// The flat map never zooms out past the whole planet nor in past its cap, and never pans
/// past the poles, so the picture fills the view. East and west it pans on and wraps.
#[test]
fn the_planet_lens_keeps_the_picture_in_view() {
    use crate::computer::Lens;
    let far = framed(Lens {
        zoom: 1.0,
        centre: Vec2::new(0.9, 0.9),
    });
    assert_eq!((far.zoom, far.centre), (WHOLE, Vec2::new(0.9, 0.0)));
    let close = framed(Lens {
        zoom: 99.0,
        centre: Vec2::new(-2.0, -0.6),
    });
    assert_eq!(close.zoom, CLOSEST);
    assert!((close.centre.y - (1.0 / CLOSEST - 0.5)).abs() < 1e-6);
    assert_eq!(close.centre.x, -2.0);
}

#[test]
fn favourites_and_waypoints_cross_the_wire() {
    use super::{from_wire, marks_of, marks_text, to_wire};
    let mut favourites = Favourites::default();
    favourites.save(BodyId::default(), DVec3::Z);
    assert_eq!(marks_of(&marks_text(&favourites.0)), favourites.0);
    assert_eq!((marks_text(&[]), marks_of("")), (String::new(), Vec::new()));
    let mut waypoints = Waypoints::default();
    toggle(
        &mut waypoints,
        Some((BodyId::default(), DVec3::new(3.0, 4.0, 0.0))),
    );
    let mine = waypoints.mine.unwrap();
    let back = from_wire(&to_wire(&mine)).unwrap();
    assert!(back.dir.distance(mine.dir) < 1e-6 && back.body == mine.body);
}

/// A name that would cross the player or a name before it, or run under the header words, is
/// left out, and a marker far enough away keeps its name.
#[test]
fn crowded_names_are_left_out() {
    use super::pins::POOL;
    use super::pins_draw::{Spot, named};
    let mut spots: [Spot; POOL] = [None; POOL];
    spots[0] = Some((Vec2::new(500.0, 358.0), 3));
    spots[1] = Some((Vec2::new(503.0, 359.0), 4));
    spots[2] = Some((Vec2::new(520.0, 362.0), 8));
    spots[3] = Some((Vec2::new(500.0, 420.0), 4));
    let shown = named(&spots, 720.0);
    assert_eq!(shown[..4], [true, false, false, true]);
    spots[0] = None;
    assert_eq!(named(&spots, 720.0)[..4], [false, true, false, true]);
    spots[3] = Some((Vec2::new(500.0, 10.0), 4));
    assert!(!named(&spots, 720.0)[3]);
}
