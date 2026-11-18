//! Poses off the ground without wings: the jump, the fall, and the upright fall on the boot boosters.

use std::f32::consts::PI;

use super::AvatarPose;
use super::clock::Clock;
use super::pose::{Core, Limb, Pose};
use crate::player::wrap;

/// Steepest bank and flight path pitch the poses follow, in radians.
pub const STEER_LIMIT: (f32, f32) = (1.0, 0.8);

/// Returns `pitch` on the half turn around the horizon. In a loop a pitch passes the vertical and
/// keeps counting; a pose that only leans with it, the head and the arms care where it points.
pub fn level(pitch: f32) -> f32 {
    match pitch.abs() > PI {
        true => wrap(pitch),
        false => pitch,
    }
}

/// Returns the rising pose: one knee tucked, the other leg trailing, arms out for balance.
pub fn jump(input: &AvatarPose) -> Pose {
    let rise = (input.vertical_speed / 8.0).clamp(0.0, 1.0);
    Pose::default()
        .with(Core::Pitch, 0.06)
        .with(Core::Spine, 0.05)
        .with(Core::HeadPitch, 0.15 * rise)
        .pair(Limb::Hip, 0.35 + 0.3 * rise, -0.2)
        .pair(Limb::Knee, 0.75 + 0.3 * rise, 0.4)
        .both(Limb::Ankle, 0.35)
        .pair(Limb::Shoulder, -0.35, 0.45)
        .both(Limb::ArmOut, 0.35)
        .both(Limb::Elbow, 0.6)
}

/// Returns the falling pose: arms out and up, legs trailing and pedalling slowly.
pub fn fall(clock: &Clock) -> Pose {
    let (pedal, flail) = (clock.wave(5.0, 0.0), clock.wave(7.0, 0.0));
    Pose::default()
        .with(Core::Pitch, 0.12)
        .with(Core::Spine, 0.08)
        .with(Core::HeadPitch, -0.25)
        .pair(Limb::Hip, -0.1 + 0.14 * pedal, -0.1 - 0.14 * pedal)
        .pair(Limb::Knee, 0.6 + 0.15 * pedal, 0.6 - 0.15 * pedal)
        .both(Limb::LegOut, 0.12)
        .both(Limb::Ankle, 0.35)
        .both(Limb::Shoulder, 0.2)
        .pair(Limb::ArmOut, 1.15 + 0.08 * flail, 1.15 - 0.08 * flail)
        .both(Limb::Elbow, 0.35)
}

/// Returns the pose on the boots alone: upright, arms slightly out, knees bent, soles facing down.
///
/// The body leans a little into the turn and with the flight path, and trembles with the thrust.
pub fn boot_fall(input: &AvatarPose, clock: &Clock) -> Pose {
    let shake = 0.012 * input.thrust.clamp(0.0, 1.0) * clock.wave(31.0, 0.0);
    let bank = input.bank.clamp(-STEER_LIMIT.0, STEER_LIMIT.0);
    let (hip, knee) = (0.32, 0.62);
    Pose::default()
        .with(
            Core::Pitch,
            -0.04 - 0.15 * level(input.pitch).clamp(-STEER_LIMIT.1, STEER_LIMIT.1),
        )
        .with(Core::Roll, 0.4 * bank)
        .with(Core::Spine, 0.04 + shake)
        .with(Core::HeadPitch, -0.1)
        .both(Limb::Hip, hip + shake)
        .both(Limb::Knee, knee)
        .both(Limb::Ankle, hip - knee)
        .both(Limb::LegOut, 0.06)
        .both(Limb::Shoulder, 0.1)
        .both(Limb::ArmOut, 0.45 + shake)
        .both(Limb::Elbow, 0.3)
}
