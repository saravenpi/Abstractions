//! The shore: a body that walks keeps to dry ground, and water it stands in slows it. A fleeing walker
//! ran straight away from what it fled, into the sea or a pond, and on along the bed under the water:
//! from above it ran into the ground, or glided through the water as if it swam.

use bevy::math::{Vec2, Vec3, Vec3Swizzles};

use super::tune::{SHORE_AHEAD, WADE};
use crate::life::body::{Body, Locomotion};

/// Returns the horizontal velocity `flat` of a body of `body` at `at` kept out of the water over
/// `ground`, whose surface stands `sea` metres high in its frame. Where the ground a stride and
/// [`SHORE_AHEAD`] seconds of its speed ahead lies under the water, the part of the velocity down
/// towards the water is taken out, so the body runs along the shore and stops on it at any speed;
/// where the shore gives no way down, it stops. A body in the water may still walk out of it.
fn ashore(flat: Vec2, at: Vec3, body: &Body, ground: &dyn Fn(Vec2) -> f32, sea: f32) -> Vec2 {
    let reach = body.height.max(0.1) + flat.length() * SHORE_AHEAD;
    let ahead = at.xz() + flat.normalize_or_zero() * reach;
    if flat.length() < 1e-4 || ground(ahead) >= sea {
        return flat;
    }
    let span = 0.5 * reach;
    let rise = |d: Vec2| ground(ahead + d * span) - ground(ahead - d * span);
    let down = -Vec2::new(rise(Vec2::X), rise(Vec2::Y));
    match down.try_normalize() {
        Some(down) => flat - down * flat.dot(down).max(0.0),
        None => Vec2::ZERO,
    }
}

/// Returns `velocity` of a body of `body` at `at` over `ground`, the surface of the sea and the ponds
/// standing `sea` metres high in its frame: kept out of the water (see [`ashore`]), and slowed by the
/// water it stands in, by [`WADE`] of its speed once the water reaches its hips. Only the bodies that
/// move on the ground keep to the shore and wade; swimmers, flyers and swarms go as they please.
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
    if !walks || !sea.is_finite() {
        return velocity;
    }
    let legs = body.height.max(0.1);
    let kept = ashore(velocity.xz(), at, body, ground, sea);
    let depth = (sea - ground(at.xz())).max(0.0);
    let slow = 1.0 - WADE * (depth / legs).min(1.0);
    Vec3::new(kept.x * slow, velocity.y, kept.y * slow)
}
