//! The shore: a body that walks keeps to dry ground. A fleeing walker ran straight away from what it
//! fled, into the sea, and on along its bed under the water: from above it ran into the ground.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::tune::SHORE_AHEAD;
use crate::life::body::{Body, Locomotion};

/// Returns `velocity` of a body of `body` at `at` kept on dry ground over `ground`, the sea surface
/// standing `sea` metres high in its frame. Where the ground a stride and [`SHORE_AHEAD`] seconds of its
/// speed ahead lies under the sea, the part of the velocity down towards the water is taken out, so the
/// body runs along the shore and stops on it at any speed; where
/// the shore gives no way down, it stops. A body already in the water may still walk out of it. Only
/// the bodies that move on the ground keep to the shore.
pub fn shore(
    velocity: Vec3,
    at: Vec3,
    body: &Body,
    ground: &dyn Fn(Vec2) -> f32,
    sea: f32,
) -> Vec3 {
    let walks = matches!(
        body.locomotion,
        Locomotion::Walk | Locomotion::Slither | Locomotion::Ooze | Locomotion::Tumble
    );
    let flat = velocity.xz();
    let reach = body.height.max(0.1) + flat.length() * SHORE_AHEAD;
    let ahead = at.xz() + flat.normalize_or_zero() * reach;
    if !walks || flat.length() < 1e-4 || ground(ahead) >= sea {
        return velocity;
    }
    let span = 0.5 * reach;
    let rise = |d: Vec2| ground(ahead + d * span) - ground(ahead - d * span);
    let down = -Vec2::new(rise(Vec2::X), rise(Vec2::Y));
    let kept = match down.try_normalize() {
        Some(down) => flat - down * flat.dot(down).max(0.0),
        None => Vec2::ZERO,
    };
    Vec3::new(kept.x, velocity.y, kept.y)
}
