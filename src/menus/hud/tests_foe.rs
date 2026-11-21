//! Tests of the strip of the animal fought: it hangs over the head of the animal on screen for
//! several camera poses, is absent when the animal is behind the camera or off screen, and never
//! sits over the crosshair.

use bevy::prelude::*;

use super::foe::{hung, over};

/// The screen of the tests in interface pixels, and its vertical field of view in radians.
const LENS: (f32, Vec2) = (1.2, Vec2::new(1280.0, 720.0));

/// Returns the screen rectangle of the strip hung for a head at `pos`, from its left and bottom.
fn rect(pos: Vec2) -> Rect {
    let (_, screen) = LENS;
    let (left, bottom) = hung(pos, screen);
    let (x, base) = (screen.x / 2.0 + left, screen.y / 2.0 - bottom);
    Rect::new(x, base - 28.0, x + 128.0, base)
}

/// Seen from in front, from the side, from above and from far, the head shows on screen, and the
/// strip sits centred over it with its bottom above it.
#[test]
fn the_strip_hangs_over_the_head() {
    let head = Vec3::new(0.0, 1.2, -6.0);
    let poses = [
        Transform::from_xyz(0.0, 1.6, 0.0).looking_at(Vec3::new(0.0, 1.0, -6.0), Vec3::Y),
        Transform::from_xyz(4.0, 1.6, -2.0).looking_at(Vec3::new(0.5, 0.8, -6.0), Vec3::Y),
        Transform::from_xyz(0.0, 6.0, 1.0).looking_at(Vec3::new(0.0, 0.0, -7.0), Vec3::Y),
        Transform::from_xyz(-2.0, 2.0, 30.0).looking_at(head, Vec3::Y),
    ];
    for pose in poses {
        let pos = over(head, &GlobalTransform::from(pose), LENS).expect("the head is on screen");
        let strip = rect(pos);
        assert!(
            strip.max.y <= pos.y + 2.0,
            "{pose:?}: {strip:?} not above {pos}"
        );
        assert!(
            (strip.center().x - pos.x).abs() <= 2.0,
            "{pose:?}: {strip:?} off {pos}"
        );
    }
}

/// An animal behind the camera, or far off to the side of the view, has no strip.
#[test]
fn the_strip_is_absent_off_screen() {
    let camera = GlobalTransform::from(Transform::from_xyz(0.0, 1.6, 0.0));
    assert!(over(Vec3::new(0.0, 1.2, 6.0), &camera, LENS).is_none());
    assert!(over(Vec3::new(40.0, 1.2, -2.0), &camera, LENS).is_none());
    assert!(over(Vec3::new(0.0, 1.2, -6.0), &camera, LENS).is_some());
}

/// A head under the crosshair puts the strip above the ticks, not over them.
#[test]
fn the_strip_keeps_off_the_crosshair() {
    let (_, screen) = LENS;
    for below in [0.0, 10.0, 30.0] {
        let strip = rect(screen / 2.0 + Vec2::new(5.0, below));
        let cross = Rect::from_center_half_size(screen / 2.0, Vec2::splat(24.0));
        assert!(strip.intersect(cross).is_empty(), "{below}: {strip:?}");
    }
}
