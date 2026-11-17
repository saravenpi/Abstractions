//! Poses with the feet on the ground: standing, striding, landing, and sitting in the ship.

use std::f32::consts::TAU;

use super::AvatarPose;
use super::clock::{Clock, ramp};
use super::gait::{Foot, bob, foot, leg_ik, run};
use super::pose::{Core, Limb, Pose};
use super::tune::{ANKLE_RISE, HIP, LEAN, PELVIS, SHIN, SINK, SOLE_Y, THIGH};

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
}

/// Returns the hip swing, knee bend and ankle pitch that put one ankle where `foot` says.
///
/// The hip sits under the leaning pelvis, the leg is solved in the frame of the pelvis and the
/// boot is kept level while it is planted. In the air the toes dip as far as the lift allows.
fn leg(stance: &Stance, foot: &Foot) -> [f32; 3] {
    let (sin, cos) = stance.lean.sin_cos();
    let hip = (PELVIS[1] + stance.lift + HIP[1] * cos, -HIP[1] * sin);
    let (y, z) = (SOLE_Y + ANKLE_RISE + foot.lift - hip.0, -foot.ahead - hip.1);
    let (swing, knee) = leg_ik(-(y * sin + z * cos), -(y * cos - z * sin));
    let dip = (foot.lift / TOE_REACH).min(0.85).asin();
    [swing, knee, swing - knee - stance.lean + dip]
}

/// Returns the pose of a pelvis held by `stance`, with both legs reaching their feet.
pub fn planted(stance: &Stance) -> Pose {
    let [left, right] = stance.feet.map(|foot| leg(stance, &foot));
    let lean_out = stance.sway / (THIGH + SHIN);
    Pose::default()
        .with(Core::Lift, stance.lift)
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

/// Returns the walking and running pose: legs on the cycle, arms swinging against them, a lean and a bob.
pub fn stride(input: &AvatarPose, clock: &Clock) -> Pose {
    let (speed, pace) = (input.speed, run(input.speed));
    let stance = Stance {
        lift: -SINK.1 + bob(clock.phase, speed) - 0.3 * clock.crouch,
        lean: LEAN.0 + LEAN.1 * speed.abs(),
        sway: 0.0,
        feet: [foot(clock.phase, speed), foot(clock.phase + 0.5, speed)],
    };
    let beat = (TAU * (clock.phase - 0.08)).cos();
    let swing = 0.3 * beat;
    planted(&stance)
        .with(Core::Spine, 0.03 + 0.03 * pace)
        .with(Core::Twist, 0.1 * beat)
        .with(Core::HeadPitch, 0.5 * stance.lean)
        .pair(
            Limb::Shoulder,
            -0.3 - 0.05 * pace - swing,
            -0.3 - 0.05 * pace + swing,
        )
        .both(Limb::ArmOut, 0.08)
        .both(Limb::Elbow, 0.3 + 0.35 * pace)
}

/// Returns the landing pose: the standing pose braced, arms out. The crouch itself comes from the clock.
pub fn land(clock: &Clock) -> Pose {
    idle(clock)
        .with(Core::Spine, 0.16)
        .with(Core::HeadYaw, 0.0)
        .both(Limb::ArmOut, 0.18)
        .both(Limb::Shoulder, 0.15)
        .both(Limb::Elbow, 0.55)
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
