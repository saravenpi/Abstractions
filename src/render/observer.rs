//! The observer: where the camera is in its star system and how system axes map to render axes.

use std::f64::consts::TAU;

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use crate::space::{Anchor, Frame};
use crate::universe::{PlanetSpec, StarSpec};
use crate::world::SEA_LEVEL;

/// Latitude in radians the legacy flat world stands at, which leans the sun path the way the old sky did.
const FLAT_LATITUDE: f64 = 0.5;

/// The camera seen from its star system: an inertial frame with the star at the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observer {
    /// Camera position in metres from the star.
    pub eye: DVec3,
    /// Centre of the planet in metres from the star.
    pub center: DVec3,
    /// Rotation from planet-fixed axes to system axes: the turn of the planet around its +Y.
    pub spin: DQuat,
    /// Rotation from system axes to render axes.
    pub to_render: DQuat,
}

impl Default for Observer {
    /// Stands one metre above an unturned planet centre, so every direction is defined.
    fn default() -> Self {
        Self {
            eye: DVec3::new(1.0, 1.0, 0.0),
            center: DVec3::X,
            spin: DQuat::IDENTITY,
            to_render: DQuat::IDENTITY,
        }
    }
}

impl Observer {
    /// Returns the observer for a camera at render position `camera` at `time` seconds.
    ///
    /// `planet` is the planet of a surface anchor, or the nearest one in space, and `star` is its star.
    pub fn new(
        anchor: &Anchor,
        camera: Vec3,
        planet: &PlanetSpec,
        star: &StarSpec,
        time: f64,
    ) -> Self {
        let center = planet.position(time);
        let spin = DQuat::from_rotation_y(TAU * time / planet.day_seconds);
        if anchor.flat {
            let height = planet.radius + f64::from(camera.y - SEA_LEVEL);
            return Self::flat(center, spin, height);
        }
        let at = anchor.to_frame(camera);
        let (eye, to_render) = match anchor.frame {
            Frame::Surface(_) => (center + spin * at, anchor.rotation * spin.inverse()),
            Frame::System(_) => (at, anchor.rotation),
            Frame::Galaxy(_) => (at - star.pos, anchor.rotation),
        };
        Self {
            eye,
            center,
            spin,
            to_render,
        }
    }

    /// Returns the observer of the legacy flat world: a fixed spot `height` metres from the planet
    /// centre, with render +Y up, +X east and +Z south.
    fn flat(center: DVec3, spin: DQuat, height: f64) -> Self {
        let (sin, cos) = FLAT_LATITUDE.sin_cos();
        let up = DVec3::new(cos, sin, 0.0);
        let east = DVec3::Y.cross(up).normalize();
        let fixed = DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up)));
        Self {
            eye: center + spin * (up * height),
            center,
            spin,
            to_render: (spin * fixed).inverse(),
        }
    }

    /// Returns the unit vector away from the planet centre, in system axes.
    pub fn up(&self) -> DVec3 {
        (self.eye - self.center).normalize_or(DVec3::Y)
    }

    /// Returns the height in metres above the sea level of a planet of `radius` metres.
    pub fn altitude(&self, radius: f64) -> f64 {
        (self.eye - self.center).length() - radius
    }

    /// Returns the unit vector toward the star, in system axes.
    pub fn sun(&self) -> DVec3 {
        (-self.eye).normalize_or(DVec3::X)
    }

    /// Returns the time of day from the angle of the star around the spin axis: 0.0 when it rises,
    /// 0.25 when it crosses the meridian, 0.5 when it sets and 0.75 at midnight.
    pub fn phase(&self) -> f32 {
        let east = DVec3::Y.cross(self.up());
        let noon = east.cross(DVec3::Y);
        let sun = self.sun();
        let ahead = sun.dot(east).atan2(sun.dot(noon));
        (0.25 - ahead / TAU).rem_euclid(1.0) as f32
    }

    /// Returns the render position of a point `pos` metres from the star, for a camera at `camera`.
    ///
    /// The offset is taken in 64 bits first, so far points keep their direction.
    pub fn place(&self, pos: DVec3, camera: Vec3) -> Vec3 {
        camera + (self.to_render * (pos - self.eye)).as_vec3()
    }
}
