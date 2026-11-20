//! How fast a falling particle sinks: a leaf as fast as drag in the air balances its weight.

use super::kind::Kind;
use super::tune;

/// Returns the speed in metres per second a leaf of edge `size` metres sinks at in air of `thickness`
/// under `gravity` metres per second squared: drag balances weight, `sqrt(2 m g / (rho Cd A))`, with the
/// mass per area of a leaf growing with the root of its size.
pub fn leaf(size: f32, gravity: f32, thickness: f32) -> f32 {
    let mass = tune::AREAL_MASS * (size / 0.1).max(0.0).sqrt();
    let density = tune::DENSITY * thickness.max(tune::SINK.0);
    (2.0 * mass * gravity / (density * tune::DRAG))
        .sqrt()
        .min(tune::SINK.1)
}

/// Returns the fall speed in metres per second of a particle of `kind` and edge `size` metres: a leaf sinks
/// at [`leaf`], anything else at the speed of its kind.
pub fn fall(kind: Kind, size: f32, gravity: f32, thickness: f32) -> f32 {
    match kind {
        Kind::Leaf => leaf(size, gravity, thickness),
        kind => kind.motion().fall,
    }
}

#[cfg(test)]
#[path = "tests_sink.rs"]
mod tests;
