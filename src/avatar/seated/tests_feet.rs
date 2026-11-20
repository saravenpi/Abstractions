//! Tests of the boots of the pilot on the rudder pedals.

use bevy::prelude::*;

use super::{rest, shod};
use crate::avatar::pose::{Limb, Pose};
use crate::ship::cockpit::Levers;

/// The boots follow the rudder pedals: a yaw to the left pushes the left toe down and lifts the
/// right one by as much, and no yaw leaves the pose alone.
#[test]
fn the_boots_follow_the_pedals() {
    let yaw = |value: f32| Levers {
        stick: Vec3::new(0.0, value, 0.0),
        ..default()
    };
    let (ankle, base) = (
        |pose: &Pose, side: usize| pose.limb(Limb::Ankle, side),
        rest(),
    );
    assert_eq!(shod(base, &yaw(0.0)), base);
    let left = shod(base, &yaw(1.0));
    assert!(ankle(&left, 0) > ankle(&base, 0) + 0.1 && ankle(&left, 1) < ankle(&base, 1) - 0.1);
    let right = shod(base, &yaw(-1.0));
    assert!(ankle(&right, 0) < ankle(&base, 0) - 0.1 && ankle(&right, 1) > ankle(&base, 1) + 0.1);
}
