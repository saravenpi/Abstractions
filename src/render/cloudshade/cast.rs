//! The shadow of the clouds at one point of the ground, worked out on the CPU the way the ground
//! shader works it out at every pixel: the ray from the ground point toward the star runs to the
//! cloud shell and reads the very puff field the clouds are drawn from, [`density_at`], at the
//! same second, so a shadow lies under its own cloud and moves with it.

use bevy::prelude::*;

use super::tune::{OPTICAL_DEPTH, SUN_EDGE};
use crate::render::smoothstep;
use crate::weather::{CELL, density_at};

/// Everything a shadow is worked out from, in axes fixed to the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cast {
    /// Sea level radius of the planet in metres.
    pub radius: f32,
    /// Radius in metres of the shell the clouds are drawn on.
    pub shell: f32,
    /// Unit vector toward the star.
    pub sun: Vec3,
    /// Universe time in seconds, which drifts the puffs.
    pub seconds: f64,
    /// Broad cloud cover over the ground, from 0 to 1.
    pub cover: f32,
}

/// Returns where the ray from `ground` toward the unit vector `sun` leaves the sphere of radius
/// `shell` metres round the planet centre, or nothing when the ground stands above it. The root
/// is taken in the form that does not cancel, as the shader takes it.
pub fn sunward_hit(ground: Vec3, sun: Vec3, shell: f32) -> Option<Vec3> {
    let b = ground.dot(sun);
    let c = ground.length_squared() - shell * shell;
    if c >= 0.0 {
        return None;
    }
    let t = -c / (b + (b * b - c).max(0.0).sqrt()).max(1.0);
    Some(ground + sun * t)
}

impl Cast {
    /// Returns the share of the direct sun that reaches the sea level ground under the
    /// planet-fixed direction `dir`: 1 in the open, down to `exp(-OPTICAL_DEPTH)` under a full
    /// puff, and 1 again as the sun comes down to the horizon.
    pub fn through(&self, dir: Vec3) -> f32 {
        let up = dir.normalize_or(Vec3::Y);
        let Some(hit) = sunward_hit(up * self.radius, self.sun, self.shell) else {
            return 1.0;
        };
        let puff = density_at(
            hit.normalize().as_dvec3(),
            self.shell / CELL,
            self.seconds,
            self.cover,
        );
        let beam = (-OPTICAL_DEPTH * puff).exp();
        1.0 - smoothstep(SUN_EDGE.0, SUN_EDGE.1, up.dot(self.sun)) * (1.0 - beam)
    }
}
