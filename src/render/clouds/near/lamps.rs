//! The lamps near the camera as the puffs of the near cloud see them: each lights a puff by its
//! illuminance there, its power over the square of the distance, within its range and, for a
//! spot, within its cone. No shadows. At night the nav lights, the strobes and the headlight of
//! the ship glow in the mist around it.

use std::f32::consts::PI;

use bevy::camera::Exposure;
use bevy::prelude::*;

use crate::render::celestial::smoothstep;

/// Share of the light that falls on a puff that it scatters back: water droplets scatter
/// nearly all of it. Unit: share.
const ALBEDO: f32 = 0.9;

/// Metres under which a lamp is taken as that far, so a puff at a lamp does not blow out.
const NEAREST: f32 = 1.0;

/// One lamp, in render space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lamp {
    /// Where it is.
    pub at: Vec3,
    /// Its linear colour times its luminous power in lumens.
    pub power: Vec3,
    /// Metres past which it lights nothing.
    pub range: f32,
    /// For a spot: the unit vector it shines along and the cosines of its outer and inner cone.
    pub cone: Option<(Vec3, f32, f32)>,
}

impl Lamp {
    /// Returns the lamp of a point light at `at`.
    pub fn point(at: Vec3, light: &PointLight) -> Self {
        Self {
            at,
            power: rgb(light.color) * light.intensity,
            range: light.range,
            cone: None,
        }
    }

    /// Returns the lamp of a spot light at `at` shining along `along`.
    pub fn spot(at: Vec3, along: Vec3, light: &SpotLight) -> Self {
        Self {
            at,
            power: rgb(light.color) * light.intensity,
            range: light.range,
            cone: Some((along, light.outer_angle.cos(), light.inner_angle.cos())),
        }
    }

    /// Returns the illuminance in lux, per colour, this lamp gives at `point`.
    pub fn lux(&self, point: Vec3) -> Vec3 {
        let to = point - self.at;
        let distance = to.length();
        if distance >= self.range || self.power == Vec3::ZERO {
            return Vec3::ZERO;
        }
        let aim = self.cone.map_or(1.0, |(along, outer, inner)| {
            smoothstep(outer, inner, to.dot(along) / distance.max(1.0e-3))
        });
        self.power * aim / (4.0 * PI * distance.max(NEAREST).powi(2))
    }
}

/// Returns the linear red, green and blue of `color`.
fn rgb(color: Color) -> Vec3 {
    let linear = color.to_linear();
    Vec3::new(linear.red, linear.green, linear.blue)
}

/// Returns the linear colour the `lamps` add to a puff at `point` under the exposure `gain`:
/// what it scatters back of their light, `albedo * E / pi`, through the exposure of the camera,
/// as the lit ground under the same lamp is drawn: Bevy scales light by `1 / (1.2 * 2^EV100)`,
/// and the gain is how many times the metered exposure brightens over the reference one.
pub fn glow(lamps: &[Lamp], point: Vec3, gain: f32) -> Vec3 {
    let lux = lamps.iter().map(|lamp| lamp.lux(point)).sum::<Vec3>();
    let exposure = gain / (1.2 * Exposure::EV100_BLENDER.exp2());
    lux * (ALBEDO / PI * exposure)
}
