//! Tests of what is drawn of one marker: a near friend is a name alone, a reduced marker a small
//! icon, the near distance follows how the player travels, the middle of the view dims and
//! markers rank from the course down.

use super::rule::{Sticky, centre, face, look, rank, span};
use super::tests_rule::marker;
use super::tune::HOME_SPAN;
use crate::menus::MarkerKind;
use crate::menus::hud::context::Context;

/// A friend is never hidden: near and in view only the name is left, on the border the icon
/// stays, far away the icon and the words are whole.
#[test]
fn a_near_friend_is_a_name_alone() {
    let friend = marker(MarkerKind::Friend, 3);
    let near = look(&friend, 3.0, Context::OnFoot, &mut Sticky::default());
    let far = look(&friend, 200.0, Context::OnFoot, &mut Sticky::default());
    assert_eq!((near.alpha, far.alpha), (1.0, 1.0));
    let tag = face(MarkerKind::Friend, near, Some(1.0));
    assert_eq!((tag.glyph, tag.label), (0.0, 1.0));
    let border = face(MarkerKind::Friend, near, None);
    assert_eq!((border.glyph, border.label), (1.0, 1.0));
    let whole = face(MarkerKind::Friend, far, Some(1.0));
    assert_eq!((whole.glyph, whole.label, whole.small), (1.0, 1.0, false));
}

/// A reduced marker is a small icon without words, and a whole one has both.
#[test]
fn a_reduced_marker_is_a_small_icon() {
    let home = marker(MarkerKind::Home, 0);
    let near = look(&home, 45.0, Context::OnFoot, &mut Sticky::default());
    let small = face(MarkerKind::Home, near, Some(1.0));
    assert!(small.small && small.label <= 0.0 && small.glyph > 0.5);
    let far = look(&home, 300.0, Context::OnFoot, &mut Sticky::default());
    let whole = face(MarkerKind::Home, far, Some(1.0));
    assert_eq!((whole.glyph, whole.label, whole.small), (1.0, 1.0, false));
}

/// In the ship in flight a target counts as near from five times farther than on foot, gliding
/// from twice, and landed as on foot.
#[test]
fn the_near_distance_follows_how_the_player_travels() {
    let on_foot = span(MarkerKind::Home, Context::OnFoot);
    assert_eq!(on_foot, HOME_SPAN);
    assert_eq!(span(MarkerKind::Home, Context::ShipLanded), on_foot);
    assert_eq!(span(MarkerKind::Home, Context::Gliding), on_foot * 2.0);
    assert_eq!(span(MarkerKind::Home, Context::ShipAir), on_foot * 5.0);
    let home = marker(MarkerKind::Home, 0);
    let flying = look(&home, 500.0, Context::ShipAir, &mut Sticky::default());
    assert_eq!((flying.alpha, flying.detail), (1.0, 1.0));
    let landing = look(&home, 100.0, Context::ShipAir, &mut Sticky::default());
    assert_eq!(landing.alpha, 0.0);
}

/// The middle of the view dims a marker, smoothly, and leaves it whole to the side.
#[test]
fn the_middle_of_the_view_dims() {
    assert!((centre(0.0) - 0.35).abs() < 1e-6);
    assert!((centre(20.0_f32.to_radians()) - 1.0).abs() < 1e-6);
    let mut last = 0.0;
    for tenth in 0..200 {
        let now = centre((tenth as f32 * 0.1).to_radians());
        assert!(now >= last && now - last < 0.05, "{tenth}");
        last = now;
    }
}

/// Markers rank: the course, the followed pin, friends, home, the ship, other pins, herds.
#[test]
fn markers_rank_from_the_course_down() {
    let order = [
        marker(MarkerKind::Course, 0),
        marker(MarkerKind::Waypoint, 0),
        marker(MarkerKind::Friend, 3),
        marker(MarkerKind::Home, 0),
        marker(MarkerKind::Ship, 0),
        marker(MarkerKind::Favourite, 0),
        marker(MarkerKind::Herd, 0),
    ];
    let ranks: Vec<u8> = order.iter().map(rank).collect();
    assert!(ranks.windows(2).all(|pair| pair[0] < pair[1]), "{ranks:?}");
    assert_eq!(rank(&marker(MarkerKind::Waypoint, 7)), rank(&order[5]));
}
