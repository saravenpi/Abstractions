//! Computer tests: the lens of the map and the mark under the pointer.

use bevy::math::Vec2;

use super::chart::Dot;
use super::lens::{Lens, MAX_ZOOM, hit};
use super::map::SIZE;

/// Returns true when two points are less than a thousandth apart.
fn close(a: Vec2, b: Vec2) -> bool {
    a.distance(b) < 1e-3
}

/// The whole chart fills the map with north up, and a pixel maps back to its chart point
/// under any lens.
#[test]
fn the_lens_maps_chart_points_to_pixels_and_back() {
    let whole = Lens::default();
    assert_eq!(whole.to_map(Vec2::ZERO), Vec2::splat(SIZE * 0.5));
    assert_eq!(whole.to_map(Vec2::new(-1.0, 1.0)), Vec2::ZERO);
    assert_eq!(whole.to_map(Vec2::new(1.0, -1.0)), Vec2::splat(SIZE));
    let lens = Lens {
        zoom: 4.0,
        centre: Vec2::new(0.3, -0.5),
    };
    assert_eq!(lens.to_map(lens.centre), Vec2::splat(SIZE * 0.5));
    for point in [Vec2::ZERO, Vec2::new(0.4, -0.6), Vec2::new(-1.0, 1.0)] {
        assert!(close(lens.to_chart(lens.to_map(point)), point), "{point}");
    }
    let east = lens.to_map(Vec2::new(0.4, -0.5)) - lens.to_map(lens.centre);
    assert!(close(east, Vec2::new(0.1 * 4.0 * SIZE * 0.5, 0.0)));
}

/// Zooming in around a pixel keeps the chart point under that pixel, and zooming back out
/// ends on the whole chart.
#[test]
fn zooming_keeps_the_point_under_the_pointer() {
    let mut lens = Lens::default();
    for pixel in [
        Vec2::new(30.0, 200.0),
        Vec2::new(120.0, 120.0),
        Vec2::new(239.0, 1.0),
    ] {
        let held = lens.to_chart(pixel);
        let closer = lens.zoomed(pixel, 1.25);
        assert!(closer.zoom > lens.zoom);
        assert!(close(closer.to_map(held), pixel), "{pixel}");
        lens = closer;
    }
    for _ in 0..40 {
        lens = lens.zoomed(Vec2::new(10.0, 10.0), 1.25);
    }
    assert_eq!(lens.zoom, MAX_ZOOM);
    for _ in 0..40 {
        lens = lens.zoomed(Vec2::new(200.0, 50.0), 0.8);
    }
    assert_eq!(lens, Lens::default());
}

/// However far the chart is dragged, the map never shows past the edge of the chart, and
/// the whole chart cannot be dragged at all.
#[test]
fn panning_stops_at_the_edge_of_the_chart() {
    let whole = Lens::default().panned(Vec2::new(500.0, -300.0));
    assert_eq!(whole, Lens::default());
    let lens = Lens::default().zoomed(Vec2::splat(SIZE * 0.5), 4.0);
    let right = lens.panned(Vec2::new(60.0, 0.0));
    assert!(close(right.centre, Vec2::new(-0.125, 0.0)), "{right:?}");
    let far = lens.panned(Vec2::new(-9000.0, 9000.0));
    assert_eq!(far.centre, Vec2::new(0.75, 0.75));
    assert!(close(far.to_map(Vec2::new(1.0, 1.0)), Vec2::new(SIZE, 0.0)));
    let back = far.panned(Vec2::new(9000.0, -9000.0));
    assert_eq!(back.centre, Vec2::new(-0.75, -0.75));
    let wild = Lens {
        zoom: 900.0,
        centre: Vec2::new(7.0, -7.0),
    };
    let tamed = wild.clamped();
    assert_eq!(tamed.zoom, MAX_ZOOM);
    assert!(tamed.centre.abs().max_element() < 1.0);
}

/// A mark outside the map is brought inside with a margin, and a mark in view moves nothing.
#[test]
fn revealing_brings_a_mark_into_view() {
    let lens = Lens::default().zoomed(Vec2::splat(SIZE * 0.5), 8.0);
    let seen = Vec2::new(0.05, -0.05);
    assert_eq!(lens.revealing(seen), lens);
    for mark in [
        Vec2::new(0.8, 0.1),
        Vec2::new(-0.88, -0.3),
        Vec2::new(0.0, 0.88),
    ] {
        let shown = lens.revealing(mark);
        let pixel = shown.to_map(mark);
        let inside = pixel.min_element() >= 10.0 && pixel.max_element() <= SIZE - 10.0;
        assert!(inside, "{mark} is drawn at {pixel}");
        assert_eq!(shown.zoom, lens.zoom);
        assert!(close(shown.revealing(mark).centre, shown.centre));
    }
    assert_eq!(
        Lens::default().revealing(Vec2::new(0.88, -0.88)),
        Lens::default()
    );
}

/// The pointer is on the nearest row or ship within reach, never on an orbit dot, and on
/// nothing when every mark is too far.
#[test]
fn the_pointer_hits_the_nearest_mark() {
    let dots = [
        (Dot::Row(0), Vec2::ZERO),
        (Dot::Orbit, Vec2::new(0.5, 0.0)),
        (Dot::Row(1), Vec2::new(0.5, 0.05)),
        (Dot::Row(2), Vec2::new(0.6, 0.05)),
        (Dot::Ship, Vec2::new(-0.5, -0.5)),
    ];
    let whole = Lens::default();
    let on = |lens: Lens, pixel: Vec2| hit(&dots, lens, pixel, 10.0);
    assert_eq!(on(whole, Vec2::new(121.0, 119.0)), Some(Dot::Row(0)));
    assert_eq!(on(whole, Vec2::new(180.0, 120.0)), Some(Dot::Row(1)));
    assert_eq!(on(whole, Vec2::new(187.0, 114.0)), Some(Dot::Row(2)));
    assert_eq!(on(whole, Vec2::new(185.0, 114.0)), Some(Dot::Row(1)));
    assert_eq!(on(whole, Vec2::new(60.0, 185.0)), Some(Dot::Ship));
    assert_eq!(on(whole, Vec2::new(150.0, 150.0)), None);
    assert_eq!(on(whole, Vec2::new(180.0, 131.0)), None);
    let closer = whole.zoomed(whole.to_map(dots[2].1), 8.0);
    assert_eq!(on(closer, closer.to_map(dots[2].1)), Some(Dot::Row(1)));
    assert_eq!(on(closer, closer.to_map(dots[1].1)), None);
    assert_eq!(hit(&[], whole, Vec2::ZERO, 10.0), None);
}
