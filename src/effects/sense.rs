//! Samples the terrain and the weather around the camera a couple of times per second.

use bevy::prelude::*;

use super::eye::Eye;
use super::ground::Ground;
use super::shed::Shedding;
use super::sky::Sky;
use crate::space::Anchor;
use crate::weather::Weather;
use crate::world::{Biome, Surface, Terrain};

/// Seconds between two samples.
const INTERVAL: f32 = 0.5;

/// Biome and surface under the camera, what the weather drops there and what the plants nearby shed.
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Sensed {
    /// Biome under the camera.
    pub biome: Biome,
    /// Surface under the camera.
    pub surface: Surface,
    /// What the weather drops around the camera.
    pub sky: Sky,
    /// The plants within reach of the particle box.
    pub shed: Shedding,
}

impl Default for Sensed {
    /// Starts on a grassy meadow under a clear sky.
    fn default() -> Self {
        Self {
            biome: Biome::Meadow,
            surface: Surface::Grass,
            sky: Sky::default(),
            shed: Shedding::default(),
        }
    }
}

/// Refreshes the sample every half second, reading the terrain view under the camera through
/// the anchor, whichever cube face the view shows, and the weather.
pub fn sense(
    time: Res<Time>,
    place: (Res<Anchor>, Res<Terrain>, Res<Eye>),
    weather: Option<Res<Weather>>,
    mut sensed: ResMut<Sensed>,
    mut wait: Local<f32>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = INTERVAL;
    let (anchor, terrain, eye) = place;
    let at = Ground::new(&anchor, &terrain, &eye).chart(eye.at);
    sensed.biome = terrain.biome(at.x, at.z);
    sensed.surface = terrain.surface(at.x, at.z);
    sensed.sky = weather.map_or(Sky::default(), |weather| Sky::of(&weather));
}
