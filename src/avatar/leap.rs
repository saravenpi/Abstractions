//! The rise of a jump: stretched out while the body shoots up, gathered at the top.
//!
//! The pose is a function of the real climb speed, so a short hop and a long jump each show
//! what they are, and the top of the rise hands over to the fall without a seam: both are the
//! gathered body there.

use super::air::mix;
use super::clock::{Clock, ramp};
use super::pose::{Core, Limb, Pose};

/// Returns the body stretched out as it shoots up: legs long, toes pointed, arms thrown up.
fn stretched() -> Pose {
    Pose::default()
        .with(Core::Pitch, 0.04)
        .with(Core::Spine, -0.06)
        .with(Core::HeadPitch, 0.2)
        .pair(Limb::Hip, 0.14, -0.12)
        .pair(Limb::Knee, 0.18, 0.28)
        .both(Limb::Ankle, 0.6)
        .pair(Limb::Shoulder, 1.15, 0.9)
        .both(Limb::ArmOut, 0.3)
        .both(Limb::Elbow, 0.45)
}

/// Returns the body gathered at the top of its jump: knees drawn up, arms in.
pub fn gathered() -> Pose {
    Pose::default()
        .with(Core::Pitch, 0.1)
        .with(Core::Spine, 0.12)
        .pair(Limb::Hip, 0.8, 0.55)
        .pair(Limb::Knee, 1.2, 0.95)
        .both(Limb::Ankle, 0.25)
        .pair(Limb::Shoulder, 0.3, 0.2)
        .both(Limb::ArmOut, 0.5)
        .both(Limb::Elbow, 0.9)
}

/// Returns the rising pose from the real climb speed: stretched out while the body shoots up,
/// gathering its knees and arms as it slows toward the top.
pub fn jump(clock: &Clock) -> Pose {
    let rise = ramp(0.5, 4.7, clock.rise);
    mix(&gathered(), &stretched(), rise)
}
