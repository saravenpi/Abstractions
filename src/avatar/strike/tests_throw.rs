//! Tests of the throw: a spear leaves along the view, whatever way the body faces, lofted a
//! little, in the upright frame of the place the body stands on.

use bevy::prelude::*;

use super::throw::aimed;
use super::tune::LOFT;
use crate::view::Heading;

/// The spear leaves along the yaw of the view, and the turn of the body does not add to it: a
/// body turned to the view, one facing north and one turned further all throw the same way.
#[test]
fn a_spear_leaves_along_the_view() {
    for yaw in [0.0_f32, 0.7, 1.5, -2.4] {
        let heading = Heading { yaw, pitch: -LOFT };
        let wanted = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        for turned in [yaw, 0.0, yaw + 1.0] {
            let player = Transform::from_rotation(Quat::from_rotation_y(turned));
            let aim = aimed(&player, &heading);
            assert!(
                aim.distance(wanted) < 1e-5,
                "view {yaw}, body {turned}: {aim}"
            );
        }
    }
}

/// A level view throws a little above the level, and on tilted ground the throw follows the
/// local up of the body.
#[test]
fn a_throw_is_lofted_in_the_upright_frame() {
    let level = Heading {
        yaw: 0.0,
        pitch: 0.0,
    };
    let aim = aimed(&Transform::IDENTITY, &level);
    assert!((aim.y - LOFT.sin()).abs() < 1e-5 && aim.z < 0.0, "{aim}");
    let up = Vec3::new(0.3, 0.8, -0.5).normalize();
    let tilt = Quat::from_rotation_arc(Vec3::Y, up);
    let player = Transform::from_rotation(tilt * Quat::from_rotation_y(0.9));
    let flat = Heading {
        yaw: 0.9,
        pitch: -LOFT,
    };
    let aim = aimed(&player, &flat);
    assert!(aim.dot(up).abs() < 1e-5, "{aim}");
    let wanted = tilt * Quat::from_rotation_y(0.9) * Vec3::NEG_Z;
    assert!(aim.distance(wanted) < 1e-5, "{aim} {wanted}");
}
