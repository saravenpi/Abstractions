//! The skid: a body sliding along the ground after a fast shallow touchdown, on its feet and
//! leaning back against the way it slides, or on its belly with the arms out if it came in prone.

use bevy::prelude::*;

use super::tune::LAIN_SECONDS;
use crate::avatar::air::mix;
use crate::avatar::clock::{Clock, ramp};
use crate::avatar::pose::{Core, Limb, Pose};
use crate::avatar::{AvatarPose, Mode};

/// Returns the skid of a body sliding as `input` says: on its feet as far as it came in upright,
/// sitting back against the way it came in, the lead leg out and the arms back for balance; on
/// its belly as far as it came in prone, back arched, head up and arms out to the sides. The
/// legs judder with the speed. The pelvis is laid on the ground by the blend.
pub fn skid(input: &AvatarPose, clock: &Clock) -> Pose {
    let way = Vec2::new(-input.struck.z, input.struck.x).normalize_or(Vec2::X);
    let judder = 0.03 * ramp(2.0, 20.0, input.speed) * clock.wave(19.0, 0.0);
    let feet = Pose::default()
        .with(Core::Pitch, -0.5 * way.x)
        .with(Core::Roll, -0.4 * way.y)
        .with(Core::Spine, 0.35)
        .with(Core::HeadPitch, 0.25)
        .pair(Limb::Hip, 1.1 + judder, 0.45)
        .pair(Limb::Knee, 0.45, 1.3 - judder)
        .pair(Limb::Ankle, -0.25, 0.25)
        .both(Limb::LegOut, 0.15)
        .both(Limb::Shoulder, -0.45)
        .both(Limb::ArmOut, 0.9 + judder)
        .both(Limb::Elbow, 0.3);
    let belly = Pose::default()
        .with(Core::Pitch, 1.5)
        .with(Core::Spine, -0.35)
        .with(Core::HeadPitch, 1.1)
        .both(Limb::Shoulder, 0.9)
        .both(Limb::ArmOut, 1.35 + judder)
        .both(Limb::Elbow, 0.35)
        .both(Limb::Hip, -0.1)
        .both(Limb::LegOut, 0.3)
        .both(Limb::Knee, 0.4 - judder)
        .both(Limb::Ankle, 0.8);
    mix(&feet, &belly, clock.lain)
}

/// Eases how far the body lies along the ground by `dt` seconds: in the air as far as it lies in
/// the arch of a fall or prone on the gear, and held from the touchdown on, so a skid keeps the
/// way it came in while it blends out.
pub fn lie(clock: &mut Clock, input: &AvatarPose, dt: f32) {
    let goal = match input.mode {
        Mode::Jump | Mode::Fall | Mode::BootFall | Mode::Glide | Mode::Fly => {
            clock.belly.max(input.prone.clamp(0.0, 1.0))
        }
        _ => clock.lain,
    };
    let step = dt / LAIN_SECONDS;
    clock.lain += (goal - clock.lain).clamp(-step, step);
}
