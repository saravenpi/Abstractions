//! The flat water under the focus, and the mirror about it.
//!
//! The sea is a sphere, but within the reach of the mirror it is flat to the millimetre: it drops
//! by `d * d / 2R`, 7 mm at 300 m on a planet of 6,000 km. The plane is the tangent of the sea
//! under the focus, whose height the sea already works out in 64 bits.

use bevy::prelude::*;

use super::super::uniform::WaterUniform;

/// Reflectance of water seen straight down, `WATER_F0` in `water.wesl`.
#[cfg(test)]
pub const WATER_F0: f32 = 0.02;

/// A flat water surface: a point on it and the way up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    /// A point on the surface, in render space.
    pub point: Vec3,
    /// The unit normal, pointing out of the water.
    pub up: Vec3,
}

impl Plane {
    /// Returns the sea level under the focus of the water values: the focus less its height.
    /// Values that never placed a sea give a plane with no way up, which nothing is above.
    pub fn under(water: &WaterUniform) -> Self {
        let up = water.upward.truncate().normalize_or_zero();
        Self {
            point: water.eye.truncate() - up * water.eye.w,
            up,
        }
    }

    /// Returns the height in metres of `p` above the surface, negative below it.
    pub fn height(&self, p: Vec3) -> f32 {
        self.up.dot(p - self.point)
    }

    /// Returns `p` mirrored about the surface.
    pub fn mirror(&self, p: Vec3) -> Vec3 {
        p - 2.0 * self.height(p) * self.up
    }

    /// Returns the mirror about the surface as a matrix. It turns space inside out, so a camera
    /// posed by it draws with its culling inverted.
    pub fn matrix(&self) -> Mat4 {
        let n = self.up;
        let turn = |axis: Vec3| (axis - 2.0 * n.dot(axis) * n).extend(0.0);
        let shift = (2.0 * n.dot(self.point) * n).extend(1.0);
        Mat4::from_cols(turn(Vec3::X), turn(Vec3::Y), turn(Vec3::Z), shift)
    }

    /// Returns the pose of the mirror camera of `eye`: the eye mirrored about the surface. It
    /// sees what lies above the water from below, each thing where the eye sees its reflection.
    pub fn eye(&self, eye: &GlobalTransform) -> Mat4 {
        self.matrix() * eye.to_matrix()
    }

    /// Returns the near clip plane of the mirror camera of `eye`, in its view space, as Bevy's
    /// `PerspectiveProjection::near_clip_plane` takes it: the normal points from the mirror
    /// camera across the surface, and the last value is minus the height of the eye. Everything
    /// under the water, the sunk legs of a wading robot too, is clipped away.
    pub fn clip(&self, eye: &GlobalTransform) -> Vec4 {
        let normal = eye.rotation().inverse() * -self.up;
        normal.extend(-self.height(eye.translation()))
    }
}

/// Returns the share of the light that water of reflectance [`WATER_F0`] mirrors for a view whose
/// angle to the normal has the cosine `cos`: the Schlick rule of `water.wesl`. It is 2 percent
/// looking straight down and all of it at a grazing angle.
#[cfg(test)]
pub fn fresnel(cos: f32) -> f32 {
    WATER_F0 + (1.0 - WATER_F0) * (1.0 - cos.clamp(0.0, 1.0)).powi(5)
}

/// Returns whether a camera looking along `forward` with the vertical field of view `fov` in
/// radians and the width to height ratio `aspect` sees any water below the level of `up`: its
/// lowest ray, toward a corner, points under the horizon.
pub fn sees_water(forward: Vec3, up: Vec3, fov: f32, aspect: f32) -> bool {
    let corner = ((fov * 0.5).tan() * (1.0 + aspect * aspect).sqrt()).atan();
    forward.dot(up).clamp(-1.0, 1.0).asin() < corner
}
