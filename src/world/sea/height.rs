//! Heights above the sea as the water shader measures them: from the focus, whose own height is worked out
//! in 64 bits, so that no height loses its millimetres in the size of the planet.
//!
//! A height taken as the distance to the planet centre minus the radius errs by half a metre on a planet
//! the size of the Earth, ten times the height of a wave.

use bevy::math::DVec3;
use bevy::prelude::*;

use super::uniform::WaterUniform;
use crate::world::Sited;

/// The focus as the shader measures heights from it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sight {
    /// Render position of the focus.
    pub eye: Vec3,
    /// Unit vector away from the planet centre at the focus, in render axes.
    pub up: Vec3,
    /// Distance of the focus from the planet centre in metres.
    pub far: f64,
}

impl Sight {
    /// Returns the sight of a focus at render position `eye` and planet position `at`, over a sea whose
    /// seat is `sited` on the planet and posed by `pose` in render space.
    pub fn of(pose: &Transform, sited: &Sited, (eye, at): (Vec3, DVec3)) -> Self {
        let local = sited.rot.inverse() * at.normalize_or(DVec3::Y);
        Self {
            eye,
            up: (pose.rotation * local.as_vec3()).normalize_or(Vec3::Y),
            far: at.length(),
        }
    }

    /// Returns `water` measuring heights from this sight over a sea of `radius` metres: the height `a` of
    /// the focus, and `a * a + 2 * R * a`, both worked out in 64 bits.
    pub fn onto(&self, water: WaterUniform, radius: f64) -> WaterUniform {
        let high = self.far - radius;
        WaterUniform {
            eye: self.eye.extend(high as f32),
            upward: self.up.extend((high * high + 2.0 * radius * high) as f32),
            ..water
        }
    }
}

#[cfg(test)]
impl WaterUniform {
    /// Returns the height above sea level in metres of render position `world`, with the 32-bit steps of
    /// `sea_height` in `water.wesl`.
    ///
    /// The height is `(|p|^2 + 2 (R + a) p.u + a^2 + 2 R a) / (|q| + R)`, where `p` is `world` seen from the
    /// focus, `u` the way up there, `a` the height of the focus, `R` the radius and `|q|` the distance of
    /// `world` from the centre.
    pub fn sea_height(&self, world: Vec3) -> f32 {
        let p = world - self.eye.truncate();
        let radius = self.globe.w;
        let lean = 2.0 * (radius + self.eye.w) * p.dot(self.upward.truncate());
        let rise = p.dot(p) + lean + self.upward.w;
        rise / ((radius * radius + rise).max(0.0).sqrt() + radius).max(0.001)
    }
}
