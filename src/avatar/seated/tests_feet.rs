//! Tests of the boots of the pilot on the rudder pedals.

use bevy::prelude::*;

use super::{offset, rest, shod};
use crate::avatar::mesh::segment;
use crate::avatar::pose::{Limb, Pose};
use crate::avatar::rig::{Seg, globals};
use crate::ship::SEAT;
use crate::ship::cockpit::{Levers, PEDAL, PEDAL_LENGTH};

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

/// Returns how far the lowest corners of the boot of `side` in `pose` are under the top of the
/// pedal plate of that side turned as `levers` ask, over the length of the plate: positive when
/// they are sunk into the plate, negative when they are above it.
fn sunk(pose: &Pose, levers: &Levers, side: usize) -> f32 {
    let seg = [Seg::BootL, Seg::BootR][side];
    let hinge = Vec3::new(PEDAL.x * (side as f32 * 2.0 - 1.0), PEDAL.y, PEDAL.z);
    let turn = levers.pedal_turn(side);
    let (top, normal) = (hinge + turn * Vec3::Y * 0.012, turn * Vec3::Y);
    let (frame, root) = (globals(pose)[seg as usize], SEAT.as_vec3() + offset());
    let mut deepest = f32::MIN;
    for point in segment(seg).points() {
        let at = root + frame * point;
        let along = (turn.inverse() * (at - hinge)).z;
        if along < 0.0 && along > -PEDAL_LENGTH && at.y < hinge.y + 0.03 {
            deepest = deepest.max((top - at).dot(normal));
        }
    }
    deepest
}

/// The soles of the pilot stand on the rudder pedals: with the pedals at rest and pushed to
/// either end, no corner of a boot sinks into a plate by more than a centimetre.
#[test]
fn the_soles_stand_on_the_pedals() {
    for yaw in [-1.0, 0.0, 1.0] {
        let levers = Levers {
            stick: Vec3::new(0.0, yaw, 0.0),
            ..default()
        };
        let pose = shod(rest(), &levers);
        for side in 0..2 {
            let deepest = sunk(&pose, &levers, side);
            assert!(
                deepest < 0.01,
                "yaw {yaw}, side {side}: sunk by {deepest} m"
            );
        }
    }
}
