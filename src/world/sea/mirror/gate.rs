//! When the water mirror draws: only when the player asked for it and it would show something.

use bevy::prelude::*;

use super::tune::{CEILING, REACH};

/// What decides whether the mirror camera draws this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scene {
    /// Whether the settings ask for mirrored actors.
    pub want: bool,
    /// Whether water that mirrors is shown near the focus.
    pub water: bool,
    /// Height in metres of the camera above the water.
    pub height: f32,
    /// Whether the camera looks low enough to see the water.
    pub looking: bool,
    /// Distance in metres from the mirror camera to the nearest mirrored actor.
    pub nearest: f32,
}

impl Scene {
    /// Returns whether the mirror camera draws: asked for, over water that mirrors, the camera
    /// above it but not too high, looking at it, and an actor near enough to show. Off, it costs
    /// nothing on the GPU: Bevy does not even extract an inactive camera.
    pub fn pass(&self) -> bool {
        let over = self.height > 0.0 && self.height < CEILING;
        self.want && self.water && over && self.looking && self.nearest < REACH
    }
}

/// Returns the distance from the mirror camera at `eye` to the nearest of the `actors`, infinite
/// without one. An actor far above the water is far from it, as its reflection is from the eye.
pub fn nearest(eye: Vec3, actors: impl Iterator<Item = Vec3>) -> f32 {
    actors
        .map(|actor| actor.distance(eye))
        .fold(f32::INFINITY, f32::min)
}
