//! Tests of the markers over the world: where a target lands for each direction, the arrow, and
//! the pool of six.

use bevy::prelude::*;

use super::place::{MAX, choose, dimmed, place};

/// The screen the cases use, in pixels, and its safe inset.
const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);
const INSET: f32 = 44.0;
const FOV: f32 = 1.3;

/// A target straight ahead lands at the centre of the screen with no arrow.
#[test]
fn ahead_lands_at_the_centre() {
    let at = place(Vec3::NEG_Z, FOV, SCREEN, INSET);
    assert!((at.pos - SCREEN * 0.5).length() < 1e-3);
    assert_eq!(at.arrow, None);
}

/// A target 90 degrees to the left sits on the left edge of the safe rectangle, at mid height,
/// with the arrow pointing left.
#[test]
fn left_lands_on_the_left_edge() {
    let at = place(Vec3::NEG_X, FOV, SCREEN, INSET);
    assert!((at.pos.x - INSET).abs() < 1e-3, "{:?}", at.pos);
    assert!((at.pos.y - SCREEN.y * 0.5).abs() < 1e-2);
    let arrow = at.arrow.expect("an arrow");
    assert!((arrow + std::f32::consts::FRAC_PI_2).abs() < 1e-3, "{arrow}");
}

/// A target directly behind and a little up lands on the top edge, not mirrored into the view.
#[test]
fn behind_and_up_lands_on_an_edge() {
    let at = place(Vec3::new(0.0, 0.1, 1.0).normalize(), FOV, SCREEN, INSET);
    assert!(at.arrow.is_some());
    assert!((at.pos.y - INSET).abs() < 1e-2, "{:?}", at.pos);
    assert!((at.pos.x - SCREEN.x * 0.5).abs() < 1e-2);
}

/// A target behind and below sits on the bottom edge.
#[test]
fn behind_and_down_lands_on_the_bottom_edge() {
    let at = place(Vec3::new(0.0, -0.4, 1.0).normalize(), FOV, SCREEN, INSET);
    assert!((at.pos.y - (SCREEN.y - INSET)).abs() < 1e-2, "{:?}", at.pos);
}

/// A target up, left and behind lands in the upper left part of the border.
#[test]
fn behind_up_left_lands_upper_left() {
    let at = place(Vec3::new(-1.0, 1.0, 1.0).normalize(), FOV, SCREEN, INSET);
    assert!(at.pos.x < SCREEN.x * 0.5 && at.pos.y < SCREEN.y * 0.5, "{:?}", at.pos);
    assert!(at.arrow.is_some());
}

/// Whatever the direction, the icon stays inside the safe rectangle.
#[test]
fn positions_stay_inside_the_safe_rect() {
    let steps = 24;
    for i in 0..steps {
        for j in 0..steps {
            let yaw = i as f32 / steps as f32 * std::f32::consts::TAU;
            let pitch = (j as f32 / steps as f32 - 0.5) * std::f32::consts::PI;
            let dir = Vec3::new(
                yaw.sin() * pitch.cos(),
                pitch.sin(),
                -yaw.cos() * pitch.cos(),
            );
            let at = place(dir, FOV, SCREEN, INSET);
            assert!(at.pos.x >= INSET - 1e-2 && at.pos.x <= SCREEN.x - INSET + 1e-2, "{dir:?}");
            assert!(at.pos.y >= INSET - 1e-2 && at.pos.y <= SCREEN.y - INSET + 1e-2, "{dir:?}");
        }
    }
}

/// The pool never exceeds six, and the items that must show come first, then the nearest.
#[test]
fn the_pool_holds_six_and_keeps_the_course_first() {
    let items: Vec<(bool, f64)> = (0..20).map(|i| (i == 17, 100.0 - f64::from(i))).collect();
    let mut chosen = Vec::new();
    choose(&items, &mut chosen);
    assert_eq!(chosen.len(), MAX);
    assert_eq!(chosen[0], 17);
    assert_eq!(chosen[1], 19);
    choose(&items[..2], &mut chosen);
    assert_eq!(chosen.len(), 2);
}

/// A marker near the middle of the view is faint, and one off to the side is whole.
#[test]
fn a_marker_near_the_centre_is_faint() {
    assert_eq!(dimmed(0.0), 0.4);
    assert_eq!(dimmed(20.0_f32.to_radians()), 1.0);
}
