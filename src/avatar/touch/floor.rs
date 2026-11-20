//! Laying a body on the ground: the pelvis moves up or down until the lowest part of the body
//! rests on the ground under the soles, whatever way the trunk is turned.

use bevy::prelude::*;

use crate::avatar::pose::{CORE_LIMITS, Core, Pose};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::tune::{ANKLE_RISE, CHEST, HEAD, SOLE_Y};

/// Height of the toe and heel points of a boot above its sole, in meters.
const BOOT_POINT: f32 = 0.04;

/// The points of the body that can rest on the ground: a segment, a point in its frame and the
/// radius of the shell around that point, in meters.
const CONTACTS: [(Seg, [f32; 3], f32); 15] = [
    (Seg::Pelvis, [0.0, 0.0, 0.0], 0.08),
    (Seg::Torso, [0.0, CHEST.0, 0.0], 0.12),
    (Seg::Head, [0.0, HEAD.0, 0.0], 0.15),
    (Seg::UpperArmL, [0.0, 0.0, 0.0], 0.05),
    (Seg::UpperArmR, [0.0, 0.0, 0.0], 0.05),
    (Seg::ForearmL, [0.0, 0.0, 0.0], 0.046),
    (Seg::ForearmR, [0.0, 0.0, 0.0], 0.046),
    (Seg::HandL, [0.0, -0.05, 0.0], 0.04),
    (Seg::HandR, [0.0, -0.05, 0.0], 0.04),
    (Seg::ShinL, [0.0, 0.0, 0.0], 0.06),
    (Seg::ShinR, [0.0, 0.0, 0.0], 0.06),
    (
        Seg::BootL,
        [0.0, BOOT_POINT - ANKLE_RISE, -0.125],
        BOOT_POINT,
    ),
    (
        Seg::BootR,
        [0.0, BOOT_POINT - ANKLE_RISE, -0.125],
        BOOT_POINT,
    ),
    (
        Seg::BootL,
        [0.0, BOOT_POINT - ANKLE_RISE, 0.055],
        BOOT_POINT,
    ),
    (
        Seg::BootR,
        [0.0, BOOT_POINT - ANKLE_RISE, 0.055],
        BOOT_POINT,
    ),
];

/// Returns the height of the lowest point of the body in `pose`, in the frame of the player body.
pub fn lowest(pose: &Pose) -> f32 {
    let frames = globals(pose);
    CONTACTS
        .iter()
        .map(|(seg, at, radius)| (frames[*seg as usize] * Vec3::from_array(*at)).y - radius)
        .fold(f32::MAX, f32::min)
}

/// Returns `pose` with the pelvis moved `share` of the way to where the lowest point of the body
/// rests on the ground, inside the limits of the lift.
pub fn grounded(pose: Pose, share: f32) -> Pose {
    if share <= 0.0 {
        return pose;
    }
    let (low, high) = CORE_LIMITS[Core::Lift as usize];
    let lift = pose.core(Core::Lift) + share * (SOLE_Y - lowest(&pose));
    pose.with(Core::Lift, lift.clamp(low, high))
}
