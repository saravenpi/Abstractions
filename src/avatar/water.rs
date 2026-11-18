//! Poses in the water: treading and swimming at the surface, and diving head first.

use std::f32::consts::FRAC_PI_2;

use super::AvatarPose;
use super::clock::{Clock, ramp};
use super::pose::{Core, Limb, Pose};
use super::tune::DIP_SPEED;

/// Returns the legs of a flutter kick of `size` radians at the swim angle `stroke`, toes pointed.
fn kick(pose: Pose, stroke: f32, size: f32) -> Pose {
    let beat = stroke.sin();
    pose.pair(Limb::Hip, 0.05 + size * beat, 0.05 - size * beat)
        .pair(Limb::Knee, 0.3 - 0.2 * beat, 0.3 + 0.2 * beat)
        .both(Limb::Ankle, 0.8)
}

/// Returns the pose of treading water: nearly upright, hands sculling at the sides.
fn tread(clock: &Clock) -> Pose {
    let scull = (clock.stroke * 0.5).sin();
    let pose = Pose::default()
        .with(Core::Pitch, 0.3)
        .both(Limb::Shoulder, 0.55 + 0.15 * scull)
        .both(Limb::ArmOut, 0.85 + 0.2 * scull)
        .both(Limb::Elbow, 0.7);
    kick(pose, clock.stroke, 0.22)
}

/// Returns the pose of swimming ahead: body level, head up, arms reaching and pulling together.
fn crawl(clock: &Clock) -> Pose {
    let pull = 0.5 - 0.5 * (clock.stroke * 0.5).cos();
    let pose = Pose::default()
        .with(Core::Pitch, 1.25)
        .with(Core::HeadPitch, 0.75)
        .both(Limb::Shoulder, 2.8 - 1.7 * pull)
        .both(
            Limb::ArmOut,
            0.2 + 0.8 * (pull * std::f32::consts::PI).sin(),
        )
        .both(Limb::Elbow, 0.15 + 0.9 * pull);
    kick(pose, clock.stroke, 0.3)
}

/// Returns the swimming pose: treading when slow, level with a kick and a stroke when moving,
/// whichever way it moves. The blend pitches it along its way.
pub fn swim(input: &AvatarPose, clock: &Clock) -> Pose {
    let speed = input.speed.hypot(input.vertical_speed);
    let glide = ramp(DIP_SPEED.0, DIP_SPEED.1, speed);
    Pose::default()
        .plus(&tread(clock), 1.0 - glide)
        .plus(&crawl(clock), glide)
}

/// Returns the diving pose: level and head first, arms together ahead, legs kicking. The blend
/// pitches it along its way.
pub fn dive(clock: &Clock) -> Pose {
    let pose = Pose::default()
        .with(Core::Pitch, FRAC_PI_2)
        .with(Core::HeadPitch, 0.5)
        .both(Limb::Shoulder, 2.95)
        .both(Limb::ArmOut, 0.1)
        .both(Limb::Elbow, 0.1);
    kick(pose, clock.stroke, 0.24)
}
