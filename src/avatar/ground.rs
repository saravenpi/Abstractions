//! Poses with the feet on the ground: standing, striding, landing, and sitting in the ship.

use std::f32::consts::TAU;

use super::AvatarPose;
use super::air::mix;
use super::clock::{Clock, ramp};
use super::gait::{Foot, bob, foot, leg_ik, run};
use super::pose::{Core, Limb, Pose};
use super::touch::tune::{HANDS, HANDS_DOWN};
use super::tune::{
    ANKLE_RISE, CROUCH, HIP, LEAN, LEAN_GAIN, PELVIS, PUSH_OFF, SHIN, SINK, SOLE_Y, THIGH,
    TURN_BANK,
};

/// Distance from the ankle to the toes in meters, with a margin: it bounds how far lifted toes may dip.
const TOE_REACH: f32 = 0.2;

/// How the pelvis is held over two feet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Stance {
    /// Height of the pelvis above its rest height, in meters.
    pub lift: f32,
    /// Forward lean in radians.
    pub lean: f32,
    /// Sideways shift toward the right, in meters.
    pub sway: f32,
    /// Where the left and the right ankle are.
    pub feet: [Foot; 2],
    /// Roll of the pelvis into a turn, in radians: positive drops the right hip.
    pub roll: f32,
}

/// Returns the hip swing, knee bend and ankle pitch that put one ankle where `foot` says.
///
/// The hip sits under the leaning pelvis, the leg is solved in the frame of the pelvis and the
/// boot is kept level while it is planted. In the air the toes dip as far as the lift allows.
/// `rise` is how far the roll of the pelvis lifts this hip, in meters.
fn leg(stance: &Stance, foot: &Foot, rise: f32) -> [f32; 3] {
    let (sin, cos) = stance.lean.sin_cos();
    let hip = (PELVIS[1] + stance.lift + HIP[1] * cos + rise, -HIP[1] * sin);
    let (y, z) = (SOLE_Y + ANKLE_RISE + foot.lift - hip.0, -foot.ahead - hip.1);
    let (swing, knee) = leg_ik(-(y * sin + z * cos), -(y * cos - z * sin));
    let dip = (foot.lift / TOE_REACH).min(0.85).asin();
    [swing, knee, swing - knee - stance.lean + dip]
}

/// Returns the pose of a pelvis held by `stance`, with both legs reaching their feet. A rolled
/// pelvis lifts one hip and drops the other: each leg is solved from where its own hip is and
/// hangs straight down, so both feet stay on the ground.
pub fn planted(stance: &Stance) -> Pose {
    let rise = HIP[0] * stance.roll.sin();
    let left = leg(stance, &stance.feet[0], rise);
    let right = leg(stance, &stance.feet[1], -rise);
    let lean_out = stance.sway / (THIGH + SHIN) - stance.roll;
    Pose::default()
        .with(Core::Lift, stance.lift)
        .with(Core::Roll, stance.roll)
        .with(Core::Pitch, stance.lean)
        .with(Core::Sway, stance.sway)
        .pair(Limb::Hip, left[0], right[0])
        .pair(Limb::Knee, left[1], right[1])
        .pair(Limb::Ankle, left[2], right[2])
        .pair(Limb::LegOut, lean_out, -lean_out)
}

/// Returns the standing pose: a servo sway like breathing, a slow weight shift and a look around now and then.
pub fn idle(clock: &Clock) -> Pose {
    let stance = Stance {
        lift: -SINK.0 - clock.crouch,
        sway: 0.012 * clock.wave(0.31, 0.0),
        ..Stance::default()
    };
    let breath = clock.wave(1.4, 0.0);
    let glance = ramp(0.55, 0.9, clock.wave(0.17, 4.0));
    planted(&stance)
        .with(Core::Spine, 0.02 + 0.012 * breath)
        .with(Core::HeadYaw, 0.5 * glance * clock.wave(0.53, 0.0))
        .with(
            Core::HeadPitch,
            0.02 * breath + 0.1 * glance * clock.wave(0.41, 1.0),
        )
        .both(Limb::ArmOut, 0.06 + 0.012 * breath)
        .pair(
            Limb::Shoulder,
            0.03 * clock.wave(1.4, 1.0),
            0.03 * clock.wave(1.4, 1.6),
        )
        .both(Limb::Elbow, 0.16 + 0.03 * breath)
}

/// Returns how far the striding body leans forward: more the faster it goes, further into a
/// start and back into a stop, by the speed it really gains or loses.
fn lean(input: &AvatarPose) -> f32 {
    let push = (LEAN_GAIN.0 * input.gain).clamp(-LEAN_GAIN.1, LEAN_GAIN.1);
    LEAN.0 + LEAN.1 * input.speed.abs() + push * ramp(0.5, 3.0, input.speed.abs())
}

/// Returns the walking and running pose: legs on the cycle, arms swinging against them, the
/// shoulders turning against the hips, a bob, a lean with the speed it gains and a bank into
/// the turn it makes. The knees keep taking a landing in while the legs run on.
pub fn stride(input: &AvatarPose, clock: &Clock) -> Pose {
    let (speed, pace) = (input.speed, run(input.speed));
    let stance = Stance {
        lift: -SINK.1 + bob(clock.phase, speed) - 0.3 * clock.crouch,
        lean: lean(input),
        sway: 0.0,
        feet: [foot(clock.phase, speed), foot(clock.phase + 0.5, speed)],
        roll: (-TURN_BANK.0 * input.turn * speed).clamp(-TURN_BANK.1, TURN_BANK.1),
    };
    let beat = (TAU * (clock.phase - 0.08)).cos();
    let swing = (0.24 + 0.2 * pace) * beat;
    planted(&stance)
        .with(Core::Spine, 0.03 + 0.03 * pace)
        .with(Core::Twist, (0.08 + 0.08 * pace) * beat)
        .with(Core::HeadPitch, 0.5 * stance.lean)
        .pair(
            Limb::Shoulder,
            -0.3 - 0.05 * pace - swing,
            -0.3 - 0.05 * pace + swing,
        )
        .both(Limb::ArmOut, 0.08)
        .both(Limb::Elbow, 0.3 + 0.35 * pace)
}

/// Returns `base` pressed down by a hard landing as far as the knees gave under it: the pelvis
/// drops further, the trunk leans over the knees and one hand, then from harder still both, reach
/// straight down to the ground. They come up again as the knees push the body back up, whether
/// it still lands or already stands. The short dip of a jump never reaches them.
pub fn hands(base: Pose, input: &AvatarPose, clock: &Clock) -> Pose {
    let down = ((clock.crouch - PUSH_OFF.0) / (CROUCH.1 - PUSH_OFF.0)).clamp(0.0, 1.0);
    let hard = input.landed_hard;
    let hands = [ramp(HANDS.0, HANDS.1, hard), ramp(HANDS.1, 1.0, hard)].map(|h| h * down);
    if hands[0] <= 0.0 {
        return base;
    }
    let stance = Stance {
        lift: -SINK.0 - clock.crouch - HANDS_DOWN.0 * hands[0],
        lean: HANDS_DOWN.1 * hands[0],
        ..Stance::default()
    };
    let spine = 0.16 + 0.44 * hands[0];
    let arm = |reach: f32| 0.15 + (stance.lean + spine - 0.15) * reach;
    let deep = planted(&stance)
        .with(Core::Spine, spine)
        .with(Core::HeadPitch, 0.6 * hands[0])
        .pair(Limb::Shoulder, arm(hands[0]), arm(hands[1]))
        .pair(Limb::ArmOut, 0.12, 0.18 + 0.5 * (hands[0] - hands[1]))
        .pair(Limb::Elbow, 0.55 - 0.5 * hands[0], 0.55 - 0.5 * hands[1]);
    mix(&base, &deep, hands[0])
}

/// Returns the landing pose: the standing pose braced, arms out, and pressed down by a hard
/// landing, see [`hands`]. The crouch itself comes from the clock.
pub fn land(input: &AvatarPose, clock: &Clock) -> Pose {
    let braced = idle(clock)
        .with(Core::Spine, 0.16)
        .with(Core::HeadYaw, 0.0)
        .both(Limb::ArmOut, 0.18)
        .both(Limb::Shoulder, 0.15)
        .both(Limb::Elbow, 0.55);
    hands(braced, input, clock)
}

/// Returns the pose seated in the ship: thighs level, shins down, hands on the controls.
pub fn seated(clock: &Clock) -> Pose {
    Pose::default()
        .with(Core::Lift, -0.3)
        .with(Core::Spine, -0.06 + 0.008 * clock.wave(1.4, 0.0))
        .both(Limb::Hip, 1.45)
        .both(Limb::Knee, 1.4)
        .both(Limb::Ankle, 0.05)
        .both(Limb::Shoulder, 0.75)
        .both(Limb::ArmOut, 0.1)
        .both(Limb::Elbow, 0.8)
}
