//! The shadow of the clouds on the ground, worked out on the CPU: for each texel of a patch of
//! ground around the camera, the share of the direct sun that the clouds along the sun ray let
//! through. The ray runs from the ground point to the cloud shell and reads the very puff field
//! the clouds are drawn from, [`density_at`], so a shadow lies under its own cloud.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::tune::{OPTICAL_DEPTH, REACH, SIDE, SUN_EDGE};
use crate::render::smoothstep;
use crate::weather::{CELL, density_at};

/// Everything a patch is worked out from, in axes fixed to the planet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cast {
    /// Unit vector from the planet centre to the centre of the patch.
    pub centre: Vec3,
    /// Unit vector along the patch, perpendicular to `centre`.
    pub east: Vec3,
    /// Unit vector across the patch, perpendicular to both.
    pub north: Vec3,
    /// Sea level radius of the planet in metres.
    pub radius: f32,
    /// Radius in metres of the shell the clouds are drawn on.
    pub shell: f32,
    /// Unit vector toward the star.
    pub sun: Vec3,
    /// Universe time in seconds, which drifts the puffs.
    pub seconds: f64,
    /// Broad cloud cover over the patch, from 0 to 1.
    pub cover: f32,
}

impl Cast {
    /// Returns the share of the direct sun that reaches the ground `x` metres east and `y`
    /// metres north of the centre of the patch: 1 in the open, down to `exp(-OPTICAL_DEPTH)` under
    /// a full puff, and 1 again as the sun comes down to the horizon.
    pub fn through(&self, x: f32, y: f32) -> f32 {
        let flat = self.centre * self.radius + self.east * x + self.north * y;
        let ground = flat.normalize() * self.radius;
        let lift = ground.normalize().dot(self.sun);
        let b = ground.dot(self.sun);
        let c = ground.length_squared() - self.shell * self.shell;
        let hit = ground + self.sun * (-b + (b * b - c).max(0.0).sqrt());
        let dir: DVec3 = hit.normalize().as_dvec3();
        let puff = density_at(dir, self.shell / CELL, self.seconds, self.cover);
        let beam = (-OPTICAL_DEPTH * puff).exp();
        1.0 - smoothstep(SUN_EDGE.0, SUN_EDGE.1, lift) * (1.0 - beam)
    }

    /// Returns the shares of the whole patch, row by row from the south, west to east.
    pub fn grid(&self) -> Vec<f32> {
        let step = 2.0 * REACH / SIDE as f32;
        let at = |i: usize| (i as f32 + 0.5) * step - REACH;
        let rows = (0..SIDE).flat_map(|j| (0..SIDE).map(move |i| (i, j)));
        rows.map(|(i, j)| self.through(at(i), at(j))).collect()
    }
}
