//! What every creature system reads: time, sky, surroundings, the ground and random numbers.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::effects::eye::Eye;
use crate::effects::ground::Ground;
use crate::effects::kind::Ambient;
use crate::effects::rng::Rng;
use crate::effects::sense::Sensed;
use crate::render::{DayClock, daylight};
use crate::space::Anchor;
use crate::wind::Wind;
use crate::world::Terrain;

/// Resources the creature systems read.
#[derive(SystemParam)]
pub struct Env<'w> {
    time: Res<'w, Time>,
    clock: Res<'w, DayClock>,
    sensed: Res<'w, Sensed>,
    wind: Res<'w, Wind>,
    anchor: Res<'w, Anchor>,
    terrain: Res<'w, Terrain>,
    eye: Res<'w, Eye>,
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
    /// Camera position in render space.
    pub camera: Vec3,
    /// Rotation from the local frame of the camera, with +Y up, to render axes.
    pub tilt: Quat,
    /// Unit wind direction in the local frame of the camera, on its XZ plane.
    pub wind: Vec2,
    /// What surrounds the camera.
    pub ambient: Ambient,
}

/// The ground and random numbers for picking places.
pub struct Ctx<'a> {
    /// The ground around the camera, for heights along the local vertical.
    pub ground: Ground<'a>,
    /// Random numbers.
    pub rng: &'a mut Rng,
}

impl Env<'_> {
    /// Returns this frame's view of the world around the camera.
    pub fn sky(&self) -> Sky {
        Sky {
            dt: self.time.delta_secs(),
            now: self.time.elapsed_secs_wrapped(),
            camera: self.eye.at,
            tilt: self.eye.tilt,
            wind: self.wind.direction,
            ambient: Ambient {
                biome: self.sensed.biome,
                surface: self.sensed.surface,
                height: self.eye.altitude,
                daylight: daylight(self.clock.phase),
                wind: self.wind.strength,
            },
        }
    }

    /// Returns the ground around the camera of this frame.
    pub fn ground(&self) -> Ground<'_> {
        Ground::new(&self.anchor, &self.terrain, &self.eye)
    }
}
