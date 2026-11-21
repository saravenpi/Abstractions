//! Tests of the nearness of one marker: it fades out over the near band and comes back farther
//! out, and a marker the player follows is never hidden by nearness.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::rule::{Look, Sticky, face, look};
use super::tune::{ARRIVE, BAND, HIDE_SPANS, HOME_SPAN, REDUCE_SPANS, STICK};
use crate::menus::hud::context::Context;
use crate::menus::{Marker, MarkerKind, MarkerPlace};

/// Returns a marker of `kind` that belongs to `owner`.
pub fn marker(kind: MarkerKind, owner: u32) -> Marker {
    Marker {
        kind,
        place: MarkerPlace::Anchor(DVec3::ZERO),
        color: Color::WHITE,
        label: String::new(),
        owner,
    }
}

/// Walks `home` from `from` metres to `to` metres in half metre steps on foot and returns how
/// it looked at each step.
fn walk(home: &Marker, sticky: &mut Sticky, (from, to): (f64, f64)) -> Vec<(f64, Look)> {
    let steps = ((from - to).abs() * 2.0) as usize;
    let at = |i: usize| from + (to - from) * i as f64 / steps as f64;
    (0..=steps)
        .map(|i| (at(i), look(home, at(i), Context::OnFoot, sticky)))
        .collect()
}

/// Walking up to the container, the home marker is whole far away, loses its words at ten
/// lengths, fades over the band above four lengths without a jump and is gone under it.
#[test]
fn a_marker_fades_out_over_the_near_band() {
    let (home, mut sticky) = (marker(MarkerKind::Home, 0), Sticky::default());
    let (reduce, hide) = (HOME_SPAN * REDUCE_SPANS, HOME_SPAN * HIDE_SPANS);
    let seen = walk(&home, &mut sticky, (300.0, 0.0));
    for pair in seen.windows(2) {
        let ((gap, now), (_, next)) = (pair[0], pair[1]);
        assert!(
            next.alpha <= now.alpha && next.detail <= now.detail,
            "{gap}"
        );
        assert!(now.alpha - next.alpha < 0.2, "a jump at {gap} m");
        assert_eq!(now.alpha >= 1.0, gap >= hide * BAND, "{gap}");
        assert_eq!(now.alpha <= 0.0, gap <= hide, "{gap}");
        assert_eq!(now.detail >= 1.0, gap >= reduce * BAND, "{gap}");
        assert_eq!(now.detail <= 0.0, gap <= reduce, "{gap}");
    }
    assert!(sticky.hidden && sticky.reduced);
}

/// Walking away again, the marker comes back a fifth farther out than it left, so a player who
/// stands on the limit sees it neither flicker nor pop.
#[test]
fn a_marker_comes_back_farther_out() {
    let (home, mut sticky) = (marker(MarkerKind::Home, 0), Sticky::default());
    let hide = HOME_SPAN * HIDE_SPANS;
    let inward = walk(&home, &mut sticky, (60.0, 0.0));
    let outward = walk(&home, &mut sticky, (0.0, 60.0));
    for (gap, seen) in &outward {
        assert_eq!(seen.alpha <= 0.0, *gap <= hide * STICK, "{gap}");
        assert_eq!(seen.alpha >= 1.0, *gap >= hide * STICK * BAND, "{gap}");
    }
    let at = |walk: &[(f64, Look)], gap: f64| walk.iter().find(|(g, _)| *g == gap).unwrap().1;
    assert!(at(&inward, 28.0).alpha > 0.5 && at(&outward, 28.0).alpha <= 0.0);
    walk(&home, &mut sticky, (60.0, hide - 0.1));
    for gap in [hide + 0.1, hide - 0.1, hide + 0.1, hide + 1.0] {
        assert_eq!(look(&home, gap, Context::OnFoot, &mut sticky).alpha, 0.0);
    }
}

/// The course is never hidden nor reduced, at any distance, wherever the player is.
#[test]
fn a_tracked_course_is_never_hidden_by_nearness() {
    let course = marker(MarkerKind::Course, 0);
    for context in Context::ALL {
        let mut sticky = Sticky::default();
        for gap in [1e12, 1e6, 300.0, 60.0, 24.0, 8.0, 1.0, 0.0, 5.0, 1e6] {
            let seen = look(&course, gap, context, &mut sticky);
            assert_eq!((seen.alpha, seen.detail), (1.0, 1.0), "{context:?} {gap}");
            let drawn = face(MarkerKind::Course, seen, None);
            assert_eq!((drawn.glyph, drawn.label, drawn.small), (1.0, 1.0, false));
        }
    }
}

/// The pin the player follows keeps its words down to the arrival, and hides only there.
#[test]
fn a_followed_pin_hides_only_on_arrival() {
    let (pin, mut sticky) = (marker(MarkerKind::Waypoint, 0), Sticky::default());
    for gap in [2000.0, 60.0, 20.0, ARRIVE * BAND] {
        let seen = look(&pin, gap, Context::OnFoot, &mut sticky);
        assert_eq!((seen.alpha, seen.detail), (1.0, 1.0), "{gap}");
    }
    assert_eq!(look(&pin, ARRIVE, Context::OnFoot, &mut sticky).alpha, 0.0);
    assert_eq!(
        look(&pin, ARRIVE + 1.0, Context::OnFoot, &mut sticky).alpha,
        0.0
    );
    assert_eq!(
        look(&pin, ARRIVE * 2.0, Context::OnFoot, &mut sticky).alpha,
        1.0
    );
    let other = marker(MarkerKind::Waypoint, 7);
    let seen = look(&other, 12.0, Context::OnFoot, &mut Sticky::default());
    assert!(
        seen.detail <= 0.0,
        "a pin nobody follows is reduced when near"
    );
}
