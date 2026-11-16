//! What every creature system reads: time, sky, surroundings, terrain and random numbers.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::effects::kind::Ambient;
use crate::effects::rng::Rng;
use crate::effects::sense::Sensed;
use crate::render::{DayClock, daylight};
use crate::wind::Wind;
use crate::world::Terrain;

/// Resources the creature systems read.
#[derive(SystemParam)]
pub struct Env<'w> {
    time: Res<'w, Time>,
    clock: Res<'w, DayClock>,
    sensed: Res<'w, Sensed>,
    wind: Res<'w, Wind>,
    /// Terrain for ground heights.
    pub terrain: Res<'w, Terrain>,
    /// Shared random numbers.
    pub rng: ResMut<'w, Rng>,
}

/// A frame's view of the world around the camera.
#[derive(Debug, Clone, Copy)]
pub struct Sky {
    /// Seconds since the last frame.
    pub dt: f32,
    /// Wrapped seconds of game time.
    pub now: f32,
    /// Camera position.
    pub camera: Vec3,
    /// Unit wind direction on the XZ plane.
    pub wind: Vec2,
    /// What surrounds the camera.
    pub ambient: Ambient,
}

/// Terrain and random numbers for picking places.
pub struct Ctx<'a> {
    /// Terrain for ground heights.
    pub terrain: &'a Terrain,
    /// Random numbers.
    pub rng: &'a mut Rng,
}

impl Env<'_> {
    /// Returns this frame's view of the world around the camera at `camera`.
    pub fn sky(&self, camera: Vec3) -> Sky {
        Sky {
            dt: self.time.delta_secs(),
            now: self.time.elapsed_secs_wrapped(),
            camera,
            wind: self.wind.direction,
            ambient: Ambient {
                biome: self.sensed.biome,
                surface: self.sensed.surface,
                height: camera.y,
                daylight: daylight(self.clock.phase),
                wind: self.wind.strength,
            },
        }
    }
}
