//! The render layers of the water mirror. The game camera draws layer 0 only. The mirror camera
//! under the water draws [`REFLECTED`] only: the actors, which sit on both layers, and the twin
//! lights that light them without shadows. Nothing else is drawn twice.

use bevy::camera::visibility::{Layer, RenderLayers};

/// The layer the water mirror camera draws.
pub const REFLECTED: Layer = 1;

/// Returns the layers of what the game camera sees and the water mirrors: the robot, the
/// companion, the ship, other players and the animals.
pub fn reflected() -> RenderLayers {
    RenderLayers::from_layers(&[0, REFLECTED])
}

/// Returns the layers of what only the water mirror draws: the twin lights, and the head of the
/// robot in a first person view, which the camera at the eyes must not see.
pub fn mirror_only() -> RenderLayers {
    RenderLayers::layer(REFLECTED)
}

/// Returns the layers of what only the game camera sees: the camera itself is pinned to them, so
/// that no actor it rides with hands it the mirror layer.
pub fn seen_only() -> RenderLayers {
    RenderLayers::layer(0)
}
