//! How steep a slope a body can walk: the friction its feet, belly or rim hold on the ground.
//!
//! A body on a slope of angle `a` stays put while `tan(a)` is under the friction coefficient of what
//! touches the ground, so the steepest walkable slope is `atan(friction)`.

use super::{Body, Locomotion};

/// Legs of a body that walks on claws and spines and grips the ground.
const GRIPS: usize = 6;

/// Returns the steepest slope in radians `body` can walk up or stand on.
pub fn slope(body: &Body) -> f32 {
    friction(body).atan()
}

/// Returns the friction coefficient of what `body` touches the ground with: rubber-like pads about
/// 0.8, gripping claws of many legs 1.4, a scaled belly 0.4, slime 0.5. Bodies of the air and the
/// water are not held to the ground.
fn friction(body: &Body) -> f32 {
    match body.locomotion {
        Locomotion::Walk if body.legs.len() >= GRIPS => 1.4,
        Locomotion::Walk => 0.8,
        Locomotion::Slither => 0.4,
        Locomotion::Ooze | Locomotion::Pulse => 0.5,
        Locomotion::Tumble => 0.6,
        Locomotion::Fly | Locomotion::Swarm | Locomotion::Swim => f32::INFINITY,
    }
}
