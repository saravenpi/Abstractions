//! Samples the terrain under the camera a couple of times per second.

use bevy::prelude::*;

use crate::render::GameCamera;
use crate::world::{Biome, Surface, Terrain};

/// Seconds between two samples.
const INTERVAL: f32 = 0.5;

/// Biome and surface under the camera.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Sensed {
    /// Biome under the camera.
    pub biome: Biome,
    /// Surface under the camera.
    pub surface: Surface,
}

impl Default for Sensed {
    /// Starts on a grassy meadow.
    fn default() -> Self {
        Self {
            biome: Biome::Meadow,
            surface: Surface::Grass,
        }
    }
}

/// Refreshes the sample every half second.
pub fn sense(
    time: Res<Time>,
    terrain: Res<Terrain>,
    camera: Single<&GlobalTransform, With<GameCamera>>,
    mut sensed: ResMut<Sensed>,
    mut wait: Local<f32>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = INTERVAL;
    let at = camera.translation();
    sensed.biome = terrain.biome(at.x, at.z);
    sensed.surface = terrain.surface(at.x, at.z);
}
