//! Poses off the ground without wings: the fall, the skydiver arch of a fast fall, the reach
//! for the ground, and the upright fall on the boot boosters. The rise of a jump is in `leap`.

use std::f32::consts::{FRAC_PI_2, PI};

use super::AvatarPose;
use super::clock::{Clock, ramp};
use super::leap::gathered;
use super::pose::{Core, Limb, Pose};
use super::tune::{ARCH_GROUND, ARCH_SINK, FLUTTER_SPEED, REACH_TIME};
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

/// Returns the pose `share` of the way from `from` to `to`, joint by joint.
pub fn mix(from: &Pose, to: &Pose, share: f32) -> Pose {
    Pose::default().plus(from, 1.0 - share).plus(to, share)
}

/// Returns the body about to touch down: legs reaching under it, soles level, knees ready to
/// give, arms out for balance, eyes on the ground.
fn reaching() -> Pose {
    Pose::default()
        .with(Core::Pitch, 0.06)
        .with(Core::Spine, 0.1)
        .with(Core::HeadPitch, -0.3)
        .pair(Limb::Hip, 0.3, 0.14)
        .pair(Limb::Knee, 0.42, 0.3)
        .pair(Limb::Ankle, -0.12, -0.16)
        .both(Limb::LegOut, 0.07)
        .both(Limb::Shoulder, 0.35)
        .both(Limb::ArmOut, 0.75)
        .both(Limb::Elbow, 0.4)
}

/// Returns how far the legs of a body falling at `input` reach for the ground, from 0 to 1,
/// from the real time left before it: the free height under it over its fall speed.
pub fn reach(input: &AvatarPose, clock: &Clock) -> f32 {
    let sink = -clock.rise;
    let left = input.clearance.max(0.0) / sink.max(1.0);
    (1.0 - ramp(REACH_TIME.0, REACH_TIME.1, left)) * ramp(0.0, 6.0, sink)
}

/// Returns the skydiver arch of a fast fall: belly to the ground, back arched, head up, arms
/// out with the elbows bent and the knees bent in a box, every limb fluttering in the wind.
fn arch(input: &AvatarPose, clock: &Clock) -> Pose {
    let wind = 0.03 * (input.airspeed / FLUTTER_SPEED).clamp(0.0, 1.0);
    let (arm, leg) = (wind * clock.wave(13.0, 0.0), wind * clock.wave(11.0, 1.7));
    Pose::default()
        .with(Core::Pitch, FRAC_PI_2)
        .with(Core::Spine, -0.32)
        .with(Core::HeadPitch, 0.9)
        .pair(Limb::Shoulder, 0.15 + arm, 0.15 - arm)
        .both(Limb::ArmOut, 1.45)
        .pair(Limb::Elbow, 0.7 - arm, 0.7 + arm)
        .pair(Limb::Hip, -0.22 + leg, -0.22 - leg)
        .both(Limb::LegOut, 0.32)
        .pair(Limb::Knee, 0.7 - leg, 0.7 + leg)
        .both(Limb::Ankle, 0.35)
}

/// Returns how far a falling body is asked to lie in the arch, from 0 to 1: as far as it falls
/// fast, or as far as the gear still lays it flat across a fall it took over, and less in the
/// last seconds before the ground, where it turns upright to land. The clock eases the body
/// toward it, see `Clock::belly`.
pub fn belly(input: &AvatarPose, clock: &Clock) -> f32 {
    let sink = -clock.rise;
    let fast = ramp(ARCH_SINK.0, ARCH_SINK.1, sink).max(input.prone.clamp(0.0, 1.0));
    let left = input.clearance.max(0.0) / sink.max(1.0);
    fast * ramp(ARCH_GROUND.0, ARCH_GROUND.1, left)
}

/// Returns the falling pose from the real fall speed and the height left: gathered at the
/// top, then arms out and up with the legs trailing and pedalling slowly, flat in the arch
/// once the fall is fast, then the legs reaching for the ground just before it.
pub fn fall(input: &AvatarPose, clock: &Clock) -> Pose {
    let (pedal, flail) = (clock.wave(5.0, 0.0), clock.wave(7.0, 0.0));
    let dropping = Pose::default()
        .with(Core::Pitch, 0.12)
        .with(Core::Spine, 0.08)
        .with(Core::HeadPitch, -0.25)
        .pair(Limb::Hip, -0.1 + 0.14 * pedal, -0.1 - 0.14 * pedal)
        .pair(Limb::Knee, 0.6 + 0.15 * pedal, 0.6 - 0.15 * pedal)
        .both(Limb::LegOut, 0.12)
        .both(Limb::Ankle, 0.35)
        .both(Limb::Shoulder, 0.2)
        .pair(Limb::ArmOut, 1.15 + 0.08 * flail, 1.15 - 0.08 * flail)
        .both(Limb::Elbow, 0.35);
    let drop = ramp(0.5, 9.0, -clock.rise);
    let upright = mix(&gathered(), &dropping, drop);
    mix(
        &mix(&upright, &arch(input, clock), clock.belly),
        &reaching(),
        reach(input, clock),
    )
}

/// Returns the pose on the boots alone: upright, arms slightly out, knees bent, soles facing down.
///
/// The body leans a little into the turn and with the flight path, and trembles with the thrust.
/// A bank past the side leans it back toward upright, so one rolling on round never jumps.
pub fn boot_fall(input: &AvatarPose, clock: &Clock) -> Pose {
    let shake = 0.012 * input.thrust.clamp(0.0, 1.0) * clock.wave(31.0, 0.0);
    let bank = input.bank.sin().asin().clamp(-STEER_LIMIT.0, STEER_LIMIT.0);
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
