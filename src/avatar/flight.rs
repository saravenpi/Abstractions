//! Poses in the wingsuit: the glide, and powered flight with the boots firing behind.
//!
//! Both grow out of the falling pose by the real state of the gear: the arms and legs sweep
//! out as far as the wings are really spread, and the trunk tips over as far as the gear
//! really lies prone. So spreading the wings shows the spread itself, and folding them is the
//! same motion backwards.

use super::AvatarPose;
use super::air::{STEER_LIMIT, fall, mix};
use super::clock::{Clock, ramp};
use super::pose::{Core, Limb, Pose};
use super::tune::{ARCH_LOAD, FLUTTER_SPEED, PRONE, TUCK_DIVE};

/// Returns how far the real load arches the flying body, from 0 in level flight to 1 in a
/// hard pull-up.
pub fn arch(input: &AvatarPose) -> f32 {
    ramp(ARCH_LOAD.0, ARCH_LOAD.1, input.load)
}

/// Returns `limbs` with the trunk of the flying body: tipped from the falling trunk to prone
/// along the flight path, all the way round in a loop, as far as the gear lies prone, banked
/// into the turn, the back arched by the load, head up to look ahead.
fn prone(limbs: Pose, input: &AvatarPose, clock: &Clock) -> Pose {
    let lie = input.prone.clamp(0.0, 1.0);
    let upright = fall(input, clock);
    let toward = |joint: Core, flat: f32| upright.core(joint) + (flat - upright.core(joint)) * lie;
    let bank = input.bank.clamp(-STEER_LIMIT.0, STEER_LIMIT.0);
    limbs
        .with(Core::Pitch, toward(Core::Pitch, PRONE - input.pitch))
        .with(Core::Roll, bank * (0.4 + 0.6 * lie))
        .with(Core::Spine, toward(Core::Spine, -0.08 - 0.22 * arch(input)))
        .with(Core::HeadPitch, toward(Core::HeadPitch, 0.95))
}

/// Returns the pitch of the boots: toes pointed along the body in a glide, soles turned back
/// as soon as the boosters really push.
fn ankle(input: &AvatarPose) -> f32 {
    0.9 * (1.0 - ramp(0.0, 0.4, input.thrust))
}

/// Returns the gliding pose: prone, arms and legs spread to stretch the fabric, trembling
/// with the airspeed, swept back and arched under load, tucked in a steep dive.
pub fn glide(input: &AvatarPose, clock: &Clock) -> Pose {
    let buffet = 0.02 * (input.airspeed / FLUTTER_SPEED).clamp(0.0, 1.0) * clock.wave(17.0, 0.0);
    let (pull, tuck) = (
        arch(input),
        ramp(TUCK_DIVE.0, TUCK_DIVE.1, -input.pitch.sin()),
    );
    let spread = Pose::default()
        .both(Limb::Shoulder, -0.1 + buffet - 0.35 * pull)
        .both(Limb::ArmOut, 1.2 - 0.35 * tuck)
        .both(Limb::Elbow, 0.08 + 0.25 * tuck)
        .both(Limb::Hip, -0.05 - buffet - 0.15 * pull)
        .both(Limb::LegOut, 0.38 - 0.18 * tuck)
        .both(Limb::Knee, 0.06)
        .both(Limb::Ankle, ankle(input));
    let open = input.wings.clamp(0.0, 1.0);
    prone(mix(&fall(input, clock), &spread, open), input, clock)
}

/// Returns the powered flight pose: prone, arms swept back along the body, legs together,
/// soles facing back, arched under load.
pub fn fly(input: &AvatarPose, clock: &Clock) -> Pose {
    let shake = 0.01 * input.thrust.clamp(0.0, 1.0) * clock.wave(31.0, 0.0);
    let pull = arch(input);
    let swept = Pose::default()
        .both(Limb::Shoulder, -0.12 - 0.3 * pull)
        .both(Limb::ArmOut, 0.3)
        .both(Limb::Elbow, 0.1)
        .both(Limb::Hip, shake - 0.12 * pull)
        .both(Limb::LegOut, 0.04)
        .both(Limb::Knee, 0.05 + shake.abs())
        .both(Limb::Ankle, ankle(input));
    let open = input.wings.clamp(0.0, 1.0);
    prone(mix(&fall(input, clock), &swept, open), input, clock)
}
