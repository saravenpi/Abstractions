//! The shove: the robot pushes a lighter animal it walks into aside, and is stopped by a heavier one,
//! whose body it meets as a capsule (see `super::bulk`).

use bevy::math::{Vec3, Vec3Swizzles};

use super::bulk::capsule;
use super::tune::SHOVE;
use crate::life::body::Body;

/// Metres of the body of the robot around its middle: `crate::player::BODY_RADIUS`.
const ROBOT_RADIUS: f32 = 0.35;

/// Returns `velocity` of an animal of `body` at `at`, its frame, with the robot standing at `robot`:
/// when the animal is `light`er than the robot and the robot touches it, a push of up to [`SHOVE`]
/// metres per second away from the robot is added, the more the deeper the robot stands in its body.
pub fn aside(velocity: Vec3, at: Vec3, (robot, light): (Vec3, bool), body: &Body) -> Vec3 {
    let (radius, long) = capsule(body.length, body.height);
    let reach = radius + 0.5 * long + ROBOT_RADIUS;
    let gap = (at - robot).xz();
    if !light || gap.length() >= reach {
        return velocity;
    }
    let away = gap.normalize_or(bevy::math::Vec2::X);
    let push = away * SHOVE * (1.0 - gap.length() / reach);
    velocity + Vec3::new(push.x, 0.0, push.y)
}
