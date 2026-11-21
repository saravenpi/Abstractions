//! The drift of the look angles behind the direction of travel, and its limits.

use bevy::prelude::*;

use crate::view::aim::Aim;
use crate::view::basis::Stance;
use crate::view::{Kind, Look, Moment, Target, ViewMode, ViewPrefs};

/// Returns the yaw of a third person view left alone for `seconds` behind a body of `kind`
/// that moves ahead and to its right at 20 metres per second.
fn left_alone(kind: Kind, seconds: f32) -> f32 {
    let target = Target {
        velocity: Vec3::new(1.0, 0.0, -1.0).normalize() * 20.0,
        kind,
        ..default()
    };
    let moment = Moment {
        target,
        look: Look::default(),
        prefs: ViewPrefs {
            on_foot: ViewMode::Third,
            ..default()
        },
        fov: 1.3,
        dt: 1.0 / 60.0,
    };
    let stance = Stance::of(&target, moment.dt, false);
    let mut aim = Aim::default();
    for _ in 0..(seconds * 60.0) as usize {
        aim.step(&moment, &stance, 1.0);
    }
    aim.heading.yaw
}

/// The rig drifts behind the direction of travel of a body on foot once the camera has
/// been left alone, and never behind a flying body, whose free look is the player module's.
#[test]
fn the_rig_does_not_follow_a_flying_body() {
    assert_eq!(left_alone(Kind::OnFoot, 1.0), 0.0);
    assert!(left_alone(Kind::OnFoot, 2.4).abs() > 0.2);
    for seconds in [1.0, 2.4, 10.0] {
        assert_eq!(left_alone(Kind::Flying, seconds), 0.0, "{seconds}");
    }
}
