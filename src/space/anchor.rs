//! The anchor: the one frame and origin everything rendered is placed against.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use crate::universe::{BodyId, GalaxyId, StarId};
use crate::world::Terrain;
use crate::world::globe::Chart;

/// A coordinate frame positions are expressed in. All frames use metres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Frame {
    /// Fixed to a planet and turning with it: origin at the planet centre, +Y its spin axis.
    Surface(BodyId),
    /// Inertial, origin at a star, planets orbiting on the XZ plane.
    System(StarId),
    /// Inertial, origin at a galaxy centre.
    Galaxy(GalaxyId),
}

/// Maps 64-bit frame coordinates to the 32-bit coordinates of `Transform` and back.
///
/// `render = rotation * (frame - origin)`.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    /// Frame the origin and every [`super::Placed`] pose are expressed in.
    pub frame: Frame,
    /// Frame position in metres that renders at the render origin.
    pub origin: DVec3,
    /// Rotation from frame axes to render axes.
    pub rotation: DQuat,
    /// True while the world is the legacy flat plane: up is +Y everywhere and altitude is `y`.
    pub flat: bool,
}

impl Anchor {
    /// Returns the anchor of a surface frame in which the chart position `at` of `terrain`
    /// renders at `at` itself, with render +Y up, +X along chart x and +Z along chart z there.
    pub fn surface(body: BodyId, terrain: &Terrain, at: Vec3) -> Self {
        let frame = Frame::Surface(body);
        if matches!(terrain.chart(), Chart::Flat) {
            return Self {
                frame,
                origin: DVec3::ZERO,
                rotation: DQuat::IDENTITY,
                flat: true,
            };
        }
        let up = terrain.up(at.x, at.z);
        let east = terrain.chart().east(at.x, at.z);
        let to_frame = DQuat::from_mat3(&DMat3::from_cols(east, up, east.cross(up)));
        Self {
            frame,
            origin: terrain.embed(at.x, at.y, at.z) - to_frame * at.as_dvec3(),
            rotation: to_frame.inverse(),
            flat: false,
        }
    }

    /// Returns the render position of a frame position.
    pub fn to_render(&self, pos: DVec3) -> Vec3 {
        (self.rotation * (pos - self.origin)).as_vec3()
    }

    /// Returns the frame position of a render position.
    pub fn to_frame(&self, pos: Vec3) -> DVec3 {
        self.origin + self.rotation.inverse() * pos.as_dvec3()
    }

    /// Returns the render rotation of an orientation given in the frame.
    pub fn rotation_to_render(&self, rot: DQuat) -> Quat {
        (self.rotation * rot).normalize().as_quat()
    }
}
