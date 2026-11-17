//! Poses in the wingsuit: the glide, and powered flight with the boots firing behind.

use super::AvatarPose;
use super::air::STEER_LIMIT;
use super::clock::Clock;
use super::pose::{Core, Limb, Pose};
use super::tune::{FLUTTER_SPEED, PRONE};

/// Returns the prone trunk: pitched along the flight path, banked into the turn, head up to look ahead.
fn prone(input: &AvatarPose) -> Pose {
    let path = input.pitch.clamp(-STEER_LIMIT.1, STEER_LIMIT.1);
    Pose::default()
        .with(Core::Pitch, PRONE - path)
        .with(Core::Roll, input.bank.clamp(-STEER_LIMIT.0, STEER_LIMIT.0))
        .with(Core::Spine, -0.08)
        .with(Core::HeadPitch, 0.95)
}

/// Returns the gliding pose: prone, arms and legs spread to stretch the fabric, trembling with the airspeed.
pub fn glide(input: &AvatarPose, clock: &Clock) -> Pose {
    let buffet = 0.02 * (input.airspeed / FLUTTER_SPEED).clamp(0.0, 1.0) * clock.wave(17.0, 0.0);
    prone(input)
        .both(Limb::Shoulder, -0.1 + buffet)
        .both(Limb::ArmOut, 1.2)
        .both(Limb::Elbow, 0.08)
        .both(Limb::Hip, -0.05 - buffet)
        .both(Limb::LegOut, 0.38)
        .both(Limb::Knee, 0.06)
        .both(Limb::Ankle, 0.9)
}

/// Returns the powered flight pose: prone, arms swept back along the body, legs together, soles facing back.
pub fn fly(input: &AvatarPose, clock: &Clock) -> Pose {
    let shake = 0.01 * input.thrust.clamp(0.0, 1.0) * clock.wave(31.0, 0.0);
    prone(input)
        .both(Limb::Shoulder, -0.12)
        .both(Limb::ArmOut, 0.3)
        .both(Limb::Elbow, 0.1)
        .both(Limb::Hip, shake)
        .both(Limb::LegOut, 0.04)
        .both(Limb::Knee, 0.05 + shake.abs())
        .both(Limb::Ankle, 0.0)
}
