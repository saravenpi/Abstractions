//! Map tests: a click on the flat map selects a marker or lets go of it, a double click
//! focuses the map on it, and zooming and panning keep the place under the pointer.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tap;
use crate::computer::{DOUBLE_CLICK, Lens, SIZE};
use crate::map::level::WHOLE;
use crate::map::pins::{Pin, Pins};
use crate::map::roam::{FOCUS, focused, scaled, slid};
use crate::map::surface::Surface;

/// Two markers on the flat map.
fn pins() -> Pins {
    Pins(vec![
        Pin {
            at: Vec2::new(0.3, 0.1),
            marker: Some(0),
        },
        Pin {
            at: Vec2::new(-0.2, 0.0),
            marker: Some(1),
        },
    ])
}

/// Returns the flat map seen whole with the pointer on the marker `hover`.
fn whole(hover: Option<usize>) -> Surface {
    let lens = Lens {
        zoom: WHOLE,
        centre: Vec2::ZERO,
    };
    Surface {
        lens,
        goal: lens,
        hover,
        at: Vec2::new(40.0, 150.0),
        point: Some(DVec3::Y),
        ..default()
    }
}

/// A click selects the marker under the pointer and a slow second click keeps it selected. A
/// click on bare ground lets go of it, the one way to let go, while it picks the place.
#[test]
fn a_click_selects_a_marker_and_empty_ground_lets_go_of_it() {
    let pins = pins();
    let mut surface = whole(Some(1));
    tap(&mut surface, &pins, 1.0);
    assert_eq!(surface.chosen, Some(1));
    tap(&mut surface, &pins, 3.0);
    assert_eq!(surface.chosen, Some(1));
    (surface.hover, surface.point) = (None, Some(DVec3::Z));
    tap(&mut surface, &pins, 7.0);
    assert_eq!((surface.chosen, surface.picked), (None, Some(DVec3::Z)));
}

/// A double click on a marker focuses the map on it, and on bare ground on the place under
/// the pointer. The selection stays, and the lens eases there from where it was.
#[test]
fn a_double_click_focuses_the_marker_or_the_place() {
    let pins = pins();
    let mut surface = whole(Some(0));
    tap(&mut surface, &pins, 1.0);
    tap(&mut surface, &pins, 1.0 + DOUBLE_CLICK * 0.5);
    let wanted = Lens {
        zoom: FOCUS,
        centre: Vec2::new(0.3, 0.1),
    };
    assert_eq!(surface.goal, wanted);
    assert_eq!(focused(wanted.centre, WHOLE), wanted);
    assert!(surface.chosen == Some(0) && surface.clicked.is_none() && surface.lens.zoom == WHOLE);
    let mut ground = whole(None);
    tap(&mut ground, &pins, 5.0);
    tap(&mut ground, &pins, 5.1);
    let place = ground.lens.to_chart(ground.at);
    assert!(ground.goal.centre.distance(place) < 1e-5 && ground.goal.zoom == FOCUS);
    assert!(ground.chosen.is_none());
}

/// Zooming keeps the place under the pointer still, a drag keeps it under the pointer, the map
/// runs on east past the turn of longitude, and the poles stop it.
#[test]
fn zooming_and_panning_the_flat_map_keep_the_place_under_the_pointer() {
    let lens = Lens {
        zoom: 2.5,
        centre: Vec2::new(0.9, 0.0),
    };
    let pointer = Vec2::new(60.0, 170.0);
    let held = lens.to_chart(pointer);
    let closer = scaled(lens, pointer, 1.3);
    assert!(closer.zoom > lens.zoom && closer.to_map(held).distance(pointer) < 1e-3);
    let drag = Vec2::new(40.0, -25.0);
    let moved = slid(lens, drag);
    assert!(moved.to_map(held).distance(pointer + drag) < 1e-3);
    let far = (0..100).fold(lens, |lens, _| slid(lens, Vec2::new(-SIZE * 0.25, 0.0)));
    assert!(far.centre.x > 2.0);
    let north = slid(lens, Vec2::new(0.0, SIZE * 40.0));
    assert!((north.centre.y - (0.5 - 1.0 / lens.zoom)).abs() < 1e-6);
}
