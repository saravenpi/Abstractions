//! The home base as other modules see it: where it is on the home planet and its named points.

use bevy::math::{DMat3, DQuat, DVec3};
use bevy::prelude::*;

use super::container;
use super::tune::{CONTAINER, FOOTPRINT, PAD_HALF};
use crate::universe::BodyId;
use crate::world::globe::{Pad, Reserve};

/// The home base: a fixed place on the home planet, the same for a world seed on every load.
///
/// Every position is in metres from the centre of the home planet, in its own turning frame,
/// whatever planet is streamed. Directions are unit vectors in the same axes.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Base {
    /// The planet the base stands on.
    pub body: BodyId,
    /// Centre of the base, on the ground.
    pub centre: DVec3,
    /// The vertical at the centre. Every platform of the base is level against it.
    pub up: DVec3,
    /// The +x axis of the base: the first horizontal axis of its layout, not the compass east.
    pub east: DVec3,
    /// The -z axis of the base, at right angles to `up` and `east`.
    pub north: DVec3,
    /// Where the robot stands at the start of a new game, on the ground in front of the hatch.
    pub wake: DVec3,
    /// Centre of the surface of the landing pad.
    pub pad: DVec3,
    /// Horizontal direction the nose of a ship parked on the pad points along.
    pub heading: DVec3,
    /// Centre of the sealed hatch of the container.
    pub door: DVec3,
}

impl Base {
    /// Returns the rotation from the axes of the base (+x east, +y up, +z south) to planet axes.
    pub fn rot(&self) -> DQuat {
        DQuat::from_mat3(&DMat3::from_cols(self.east, self.up, -self.north))
    }

    /// Returns the orientation in planet axes of a ship parked on the pad: upright along the
    /// vertical of the pad, its nose (body -z) along the heading.
    pub fn pad_rot(&self) -> DQuat {
        let up = self.pad.normalize();
        let back = -(self.heading - up * self.heading.dot(up)).normalize();
        DQuat::from_mat3(&DMat3::from_cols(up.cross(back), up, back))
    }

    /// Returns the horizontal direction from the wake point to the hatch: where the robot looks
    /// when it wakes.
    pub fn look(&self) -> DVec3 {
        let (up, toward) = (self.wake.normalize(), self.door - self.wake);
        (toward - up * toward.dot(up)).normalize()
    }

    /// Returns where home is: the point above the roof of the container the operator lives in.
    /// The compass, the minimap, the map and the marker over the world all point at it, so the
    /// house icon sits on the building. The pad keeps the marker of the ship.
    pub fn home(&self) -> DVec3 {
        let turn = self.rot() * DQuat::from_rotation_y(f64::from(CONTAINER.yaw));
        self.door + turn * (container::HOME - container::DOOR).as_dvec3()
    }

    /// Returns true when the planet position `pos` lies over the footprint of the base grown by
    /// `margin` metres. Props and structure sites keep out of it.
    pub fn covers(&self, pos: DVec3, margin: f64) -> bool {
        let from = pos - self.centre;
        (from - self.up * from.dot(self.up)).length() <= FOOTPRINT + margin
    }

    /// Returns the ground the base keeps free: its footprint, and inside it the square of the landing pad where no
    /// plant of any size stands.
    pub fn reserve(&self) -> Reserve {
        Reserve {
            centre: self.centre,
            up: self.up,
            radius: FOOTPRINT,
            pad: Some(Pad {
                centre: self.pad,
                up: self.pad.normalize(),
                heading: self.heading,
                half: f64::from(PAD_HALF),
            }),
        }
    }

    /// Returns true when the planet position `pos` lies over the landing pad.
    pub fn on_pad(&self, pos: DVec3) -> bool {
        let (from, up) = (pos - self.pad, self.pad.normalize());
        let side = up.cross(self.heading);
        from.dot(self.heading).abs().max(from.dot(side).abs()) <= f64::from(PAD_HALF)
    }
}
