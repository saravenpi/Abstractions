//! Tests of the shake in the seat: under the hardest shake the eye stays where it sits, the
//! cockpit holds still in the view to well under a pixel, and only the world outside trembles.

use bevy::prelude::*;

use super::{braced, felt};
use crate::ship::shake::JOLT;

/// Angle in radians of one pixel of the 640 by 360 picture under a vertical field of view of
/// 70 degrees.
const PIXEL: f32 = 1.22 / 360.0;

/// Returns the direction in which the camera turned `rotation` at `eye` sees `point`.
fn seen(rotation: Quat, eye: Vec3, point: Vec3) -> Vec3 {
    (rotation.inverse() * (point - eye)).normalize()
}

/// Over two seconds of the shake of rough air at full strength, and of a fresh crash jolt on
/// top of it, the eye in the seat never moves and a point of the panel 80 cm ahead stays within
/// a tenth of a pixel where it was, while a point of the world a kilometre out moves by about
/// a pixel or more and, in rough air, by under half a degree.
#[test]
fn the_cockpit_holds_still_while_the_world_shakes() {
    let (eye, rotation) = (Vec3::new(3.0, 2.0, -1.0), Quat::from_rotation_y(0.4));
    let panel = eye + rotation * Vec3::new(0.1, -0.3, -0.8);
    let far = eye + rotation * Vec3::new(0.0, 0.0, -1000.0);
    for strength in [1.0, 1.0 + JOLT.0] {
        let mut moved = 0.0f32;
        for frame in 0..120 {
            let (offset, tilt) = felt(frame as f32 / 60.0, strength, true);
            assert_eq!(offset, Vec3::ZERO);
            let shaken = rotation * tilt;
            let turn = shaken * tilt * shaken.inverse();
            let hull = braced(Transform::from_translation(panel), eye, turn).translation;
            let drift = seen(shaken, eye, hull).angle_between(seen(rotation, eye, panel));
            assert!(
                drift < 0.1 * PIXEL,
                "the panel moved {drift} rad on frame {frame}"
            );
            moved = moved.max(seen(shaken, eye, far).angle_between(seen(rotation, eye, far)));
        }
        assert!(
            moved > 0.5 * PIXEL,
            "the world moved only {moved} rad at {strength}"
        );
        if strength == 1.0 {
            assert!(moved < 0.5f32.to_radians(), "the world moved {moved} rad");
        }
    }
}
