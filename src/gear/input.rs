//! What the flight gear reads each step.

use bevy::prelude::*;

use super::tune::SEA_DENSITY;

/// Share of the atmosphere height that makes one scale height, as in `ship::drag`.
const SCALE_SHARE: f32 = 0.125;

/// Everything the gear needs to know about the player and the air for one step.
///
/// Vectors are in the upright frame of the body: +Y is the local up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GearInput {
    /// Whether the wingsuit is wanted open. A level, not an edge: toggle it outside.
    pub deploy: bool,
    /// Whether the boot boosters are held on.
    pub boost: bool,
    /// Pitch stick from -1 (nose down, or forward with the boots alone) to 1 (nose up, or back).
    pub pitch: f32,
    /// Roll stick from -1 (left) to 1 (right). With the boots alone it strafes.
    pub roll: f32,
    /// Unit direction the camera looks along. Zero keeps the current heading and a level nose.
    pub look: Vec3,
    /// Whether the controller found walkable ground under the body this frame.
    pub grounded: bool,
    /// Whether the body is in water deep enough to swim.
    pub in_water: bool,
    /// Air density in kilograms per cubic metre. See [`air_density`].
    pub density: f32,
    /// Gravity in metres per second squared, a magnitude.
    pub gravity: f32,
    /// Free height under the body in metres, for the ground assist. Infinite when unknown.
    pub clearance: f32,
    /// Strength of the ground assist from 0 (off) to 1.
    pub assist: f32,
    /// Velocity of the air over the ground in metres per second. The gear flies through this
    /// air: lift, drag and airspeed all count from it. Zero is still air.
    pub wind: Vec3,
}

impl Default for GearInput {
    /// Standing in the air of the home planet at sea level with nothing held.
    fn default() -> Self {
        Self {
            deploy: false,
            boost: false,
            pitch: 0.0,
            roll: 0.0,
            look: Vec3::NEG_Z,
            grounded: false,
            in_water: false,
            density: SEA_DENSITY,
            gravity: super::tune::REF_GRAVITY,
            clearance: f32::INFINITY,
            assist: 1.0,
            wind: Vec3::ZERO,
        }
    }
}

/// Returns the air density in kilograms per cubic metre `altitude` metres above the sea of a
/// planet whose atmosphere is `atmosphere_height` metres deep.
///
/// The same exponential atmosphere as `ship::drag::density`: zero above the top.
pub fn air_density(atmosphere_height: f32, altitude: f32) -> f32 {
    if altitude >= atmosphere_height || atmosphere_height <= 0.0 {
        return 0.0;
    }
    SEA_DENSITY * (-altitude.max(0.0) / (atmosphere_height * SCALE_SHARE)).exp()
}
