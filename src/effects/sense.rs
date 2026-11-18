//! Samples the terrain, the weather and the shelter around the camera a couple of times per second.

use bevy::prelude::*;

use super::eye::Eye;
use super::ground::Ground;
use super::sky::{Sky, roofed};
use crate::space::Anchor;
use crate::weather::Weather;
use crate::world::structures::Surroundings;
use crate::world::{Biome, Surface, Terrain};

/// Seconds between two samples.
const INTERVAL: f32 = 0.5;

/// Distance in metres within which the nearest site says what a built floor belongs to.
const SITE_REACH: f32 = 80.0;

/// Depth in metres under the camera down to which a built floor counts as the one it stands on,
/// and the step of the search: the camera may float above the feet it follows.
const FLOOR_PROBE: (f32, f32) = (2.5, 0.5);

/// Biome and surface under the camera, and what the weather drops there.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Sensed {
    /// Biome under the camera.
    pub biome: Biome,
    /// Surface under the camera.
    pub surface: Surface,
    /// What the weather drops around the camera.
    pub sky: Sky,
}

impl Default for Sensed {
    /// Starts on a grassy meadow under a clear sky.
    fn default() -> Self {
        Self {
            biome: Biome::Meadow,
            surface: Surface::Grass,
            sky: Sky::default(),
        }
    }
}

/// Returns true when the camera of `eye` stands under a roof among the sites `around`.
fn sheltered(anchor: &Anchor, eye: &Eye, around: &Surroundings) -> bool {
    let up = eye.tilt * Vec3::Y;
    let near = around.nearest(anchor.to_frame(eye.at), SITE_REACH);
    let steps = (FLOOR_PROBE.0 / FLOOR_PROBE.1) as usize;
    (0..=steps).any(|step| {
        let probe = eye.at - up * FLOOR_PROBE.1 * step as f32;
        roofed(
            around.floor_at(anchor.to_frame(probe)),
            near.map(|site| site.kind),
        )
    })
}

/// Refreshes the sample every half second, reading the terrain view under the camera through
/// the anchor, whichever cube face the view shows, and the weather unless a roof is overhead.
pub fn sense(
    time: Res<Time>,
    place: (Res<Anchor>, Res<Terrain>, Res<Eye>),
    sky: (Option<Res<Weather>>, Option<Res<Surroundings>>),
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
    let (weather, around) = sky;
    let roof = around.is_some_and(|around| sheltered(&anchor, &eye, &around));
    sensed.sky = weather.map_or(Sky::default(), |weather| Sky::of(&weather, roof));
}
