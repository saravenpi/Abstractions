//! Map tests: a level fades in, the flat map opens where it was asked to look and no closer
//! than its cap, pins cross the wire, and crowded names on it are left out.

use bevy::math::{DVec2, DVec3, Vec2};

use super::fade::{FADE, shade};
use super::level::{CLOSEST, GROUND, REFERENCE_MAP, WHOLE, first, framed};
use super::raster::tile::{LEVEL_MAX, SIDE, TileId};
use super::{Places, to_chart};
use crate::universe::BodyId;

#[test]
fn a_level_fades_in_and_opens_where_asked() {
    assert_eq!(shade(0.0), 1.0);
    assert!((shade(FADE * 0.5) - 0.5).abs() < 1e-6);
    assert_eq!(shade(FADE * 2.0), 0.0);
    let whole = first(None, None);
    assert_eq!((whole.zoom, whole.centre), (WHOLE, DVec2::ZERO));
    let here = DVec3::new(0.3, 0.2, -0.9);
    let close = first(None, Some(here));
    assert_eq!(close.zoom, GROUND);
    const { assert!(GROUND < CLOSEST) };
    assert!(close.centre.distance(to_chart(here)) < 1e-6);
    let asked = first(Some(DVec3::Z), Some(here));
    assert!(asked.centre.distance(to_chart(DVec3::Z)) < 1e-6);
}

/// At the closest zoom a pixel of the finest tile takes between one and two pixels of the map
/// of a 1280 by 720 window, and the closest zoom still comes closer than the whole planet.
#[test]
fn the_closest_zoom_keeps_tile_pixels_small() {
    let across = f64::from(TileId::cols(LEVEL_MAX)) * SIDE as f64;
    let pixel = f64::from(CLOSEST * REFERENCE_MAP) / across;
    assert!((1.0..2.0).contains(&pixel), "{pixel}");
    const { assert!(CLOSEST > WHOLE) };
}

/// The flat map never zooms out past the whole planet nor in past its cap, and never pans
/// past the poles, so the picture fills the view. East and west it pans on and wraps.
#[test]
fn the_planet_lens_keeps_the_picture_in_view() {
    use super::sight::Sight;
    let far = framed(Sight {
        zoom: 1.0,
        centre: DVec2::new(0.9, 0.9),
    });
    assert_eq!((far.zoom, far.centre), (WHOLE, DVec2::new(0.9, 0.0)));
    let close = framed(Sight {
        zoom: 1.0e6,
        centre: DVec2::new(-2.0, -0.6),
    });
    assert_eq!(close.zoom, CLOSEST);
    assert!((close.centre.y - (1.0 / f64::from(CLOSEST) - 0.5)).abs() < 1e-9);
    assert_eq!(close.centre.x, -2.0);
}

/// A pin crosses the wire as the active pin of a player, and the pins ride the record.
#[test]
fn pins_cross_the_wire() {
    use super::{from_wire, marks_of, marks_text, to_wire};
    let mut places = Places::default();
    places.drop(BodyId::default(), DVec3::new(3.0, 4.0, 0.0));
    assert_eq!(marks_of(&marks_text(&places.pins)), places.pins);
    let mine = places.pins[0].clone();
    let back = from_wire(&to_wire(&mine)).unwrap();
    let (was, now) = (DVec3::from_array(mine.dir), DVec3::from_array(back.dir));
    assert!(was.distance(now) < 1e-6 && back.body == mine.body);
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
