//! The flat frame an animal moves in: a plane tangent to the planet where it was spawned, +Y up,
//! and the ground of the planet read in that plane.
//!
//! Motion works in a local flat frame with a ground height function. The frame is fixed for the life
//! of the animal, so its gait state never jumps; the ground function follows the curve of the planet
//! under the plane, so feet stay on the real ground however far the animal walks.

use bevy::math::{DMat3, DQuat, DVec3, Vec2, Vec3};

use crate::world::Terrain;

/// A flat frame on the planet: its origin on the sea level sphere and its axes, +X east, +Y up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spot {
    /// Origin in metres from the planet centre.
    pub origin: DVec3,
    /// Rotation from the axes of the frame to planet axes.
    pub rot: DQuat,
}

/// Returns the unit east tangent at unit direction `dir`, north being the spin axis, +Y: the east
/// of `crate::life::herd`.
pub fn east(dir: DVec3) -> DVec3 {
    let east = DVec3::Y.cross(dir);
    if east.length_squared() > 1e-12 {
        east.normalize()
    } else {
        DVec3::X.cross(dir).normalize()
    }
}

/// Returns the unit direction `offset` metres east and north of unit direction `dir` on a planet
/// of `radius` metres.
pub fn beside(dir: DVec3, offset: Vec2, radius: f64) -> DVec3 {
    let east = east(dir);
    let north = dir.cross(east);
    let step = east * f64::from(offset.x) + north * f64::from(offset.y);
    (dir + step / radius).normalize()
}

impl Spot {
    /// Returns the frame whose origin is at unit direction `dir` on a sphere of `radius` metres.
    pub fn at(dir: DVec3, radius: f64) -> Self {
        let up = dir.normalize();
        let east = east(up);
        Self {
            origin: up * radius,
            rot: DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up))),
        }
    }

    /// Returns planet position `pos`, in metres from the planet centre, in the frame.
    pub fn local(&self, pos: DVec3) -> Vec3 {
        (self.rot.inverse() * (pos - self.origin)).as_vec3()
    }

    /// Returns the planet position of the point `local` of the frame.
    pub fn planet(&self, local: Vec3) -> DVec3 {
        self.origin + self.rot * local.as_dvec3()
    }

    /// Returns the height in the frame of the point over `(x, z)` that lies `radius` metres from the
    /// planet centre, along the frame's vertical.
    pub fn rise(&self, at: Vec2, radius: f64) -> f32 {
        let flat = f64::from(at.length_squared());
        let r = self.origin.length();
        ((radius * radius - flat).max(0.0).sqrt() - r) as f32
    }

    /// Returns the height in the frame of the drawn ground of `terrain` over `(x, z)`, its heights read
    /// through `height` at chart positions.
    pub fn ground_by(&self, terrain: &Terrain, at: Vec2, height: impl Fn(f32, f32) -> f32) -> f32 {
        let chart = self.chart(terrain, at);
        self.rise(
            at,
            terrain.globe().radius() + f64::from(height(chart.x, chart.y)),
        )
    }

    /// Returns the height in the frame of the drawn ground of `terrain` over `(x, z)`.
    pub fn ground(&self, terrain: &Terrain, at: Vec2) -> f32 {
        self.ground_by(terrain, at, |x, z| terrain.mesh_height(x, z))
    }
}
