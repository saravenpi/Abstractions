//! Tests of the markers over the world: where a target lands for each direction, and the arrow.

use bevy::prelude::*;

use super::place::place;

/// The screen the cases use, in pixels, and the rectangle icons stay in.
const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);
const SAFE: Rect = Rect {
    min: Vec2::new(68.0, 108.0),
    max: Vec2::new(1212.0, 544.0),
};
const FOV: f32 = 1.3;

/// A target straight ahead lands at the centre of the screen with no arrow.
#[test]
fn ahead_lands_at_the_centre() {
    let at = place(Vec3::NEG_Z, FOV, SCREEN, SAFE);
    assert!((at.pos - SCREEN * 0.5).length() < 1e-3);
    assert_eq!(at.arrow, None);
}

/// A target 90 degrees to the left sits on the left edge of the safe rectangle, at mid height,
/// with the arrow pointing left.
#[test]
fn left_lands_on_the_left_edge() {
    let at = place(Vec3::NEG_X, FOV, SCREEN, SAFE);
    assert!((at.pos.x - SAFE.min.x).abs() < 1e-3, "{:?}", at.pos);
    assert!((at.pos.y - SCREEN.y * 0.5).abs() < 1e-2);
    let arrow = at.arrow.expect("an arrow");
    assert!(
        (arrow + std::f32::consts::FRAC_PI_2).abs() < 1e-3,
        "{arrow}"
    );
}

/// A target directly behind, level or a little up, lands on the bottom edge in the middle: it
/// is not mirrored into the view and never sits on the top edge, where the compass is.
#[test]
fn behind_lands_on_the_bottom_edge() {
    for up in [0.0, 0.1, 0.4] {
        let at = place(Vec3::new(0.0, up, 1.0).normalize(), FOV, SCREEN, SAFE);
        assert!(at.arrow.is_some());
        assert!((at.pos.y - SAFE.max.y).abs() < 1e-2, "{up}: {:?}", at.pos);
        assert!((at.pos.x - SCREEN.x * 0.5).abs() < 1e-2, "{up}");
    }
}

/// A target behind and below sits on the bottom edge.
#[test]
fn behind_and_down_lands_on_the_bottom_edge() {
    let at = place(Vec3::new(0.0, -0.4, 1.0).normalize(), FOV, SCREEN, SAFE);
    assert!((at.pos.y - SAFE.max.y).abs() < 1e-2, "{:?}", at.pos);
}

/// A target behind keeps the side it is on, and sinks under the middle of the screen.
#[test]
fn behind_keeps_its_side() {
    for side in [-1.0_f32, 1.0] {
        let at = place(Vec3::new(side, 0.0, 1.0).normalize(), FOV, SCREEN, SAFE);
        assert_eq!((at.pos.x - SCREEN.x * 0.5).signum(), side, "{:?}", at.pos);
        assert!(at.pos.y > SCREEN.y * 0.5, "{:?}", at.pos);
    }
}

/// A target straight overhead, neither ahead nor behind, sits on the top edge: look up.
#[test]
fn overhead_lands_on_the_top_edge() {
    let at = place(Vec3::Y, FOV, SCREEN, SAFE);
    assert!((at.pos.y - SAFE.min.y).abs() < 1e-2, "{:?}", at.pos);
}

/// The place along the border does not jump as a target passes from ahead to behind.
#[test]
fn the_border_is_continuous_across_the_side() {
    let ahead = place(Vec3::new(1.0, 0.2, -0.01).normalize(), FOV, SCREEN, SAFE);
    let behind = place(Vec3::new(1.0, 0.2, 0.01).normalize(), FOV, SCREEN, SAFE);
    assert!(
        ahead.pos.distance(behind.pos) < 12.0,
        "{ahead:?} {behind:?}"
    );
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
            let at = place(dir, FOV, SCREEN, SAFE);
            assert!(SAFE.inflate(1e-2).contains(at.pos), "{dir:?}: {:?}", at.pos);
        }
    }
}
