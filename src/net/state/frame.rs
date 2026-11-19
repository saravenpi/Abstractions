//! Every frame a pose may be in, on the wire or on screen, and how one moves into another.
//!
//! A frame is tied to the inertial frame of its star by a [`Rigid`] motion: the frame of a
//! turning planet spins and moves along its orbit, the frame around a planet only moves, the
//! frame of the star is the star frame itself, and the frame of a galaxy is shifted by where
//! the star sits in it. A pose goes out of one frame into the star frame, then back into the
//! other, at one universe time.

use std::f64::consts::TAU;

use abstractions_net::wire;
use bevy::math::{DQuat, DVec3};

use crate::space;
use crate::universe::{BodyId, GalaxyId, PlanetSpec, StarId};
use crate::world::spin;

/// A frame of the wire or of the render anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Place {
    /// Fixed to a planet or a moon and turning with it.
    Surface(BodyId),
    /// Centred on a planet or a moon, not turning.
    Around(BodyId),
    /// Inertial, origin at a star.
    System(StarId),
    /// Inertial, origin at a galaxy centre.
    Galaxy(GalaxyId),
}

impl Place {
    /// Returns how far the frame lies from the ground: 0 on a surface, then around a body, the
    /// frame of a star and the frame of a galaxy.
    pub fn depth(self) -> u8 {
        match self {
            Self::Surface(_) => 0,
            Self::Around(_) => 1,
            Self::System(_) => 2,
            Self::Galaxy(_) => 3,
        }
    }
}

impl From<wire::Frame> for Place {
    /// Returns the place of a wire frame.
    fn from(frame: wire::Frame) -> Self {
        match frame {
            wire::Frame::Surface(body) => Self::Surface(body),
            wire::Frame::Around(body) => Self::Around(body),
            wire::Frame::System(star) => Self::System(star),
        }
    }
}

impl From<space::Frame> for Place {
    /// Returns the place of a frame of the game.
    fn from(frame: space::Frame) -> Self {
        match frame {
            space::Frame::Surface(body) => Self::Surface(body),
            space::Frame::System(star) => Self::System(star),
            space::Frame::Galaxy(galaxy) => Self::Galaxy(galaxy),
        }
    }
}

/// A position, a velocity and an orientation in one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// Position in metres.
    pub pos: DVec3,
    /// Velocity in metres per second, relative to the frame.
    pub vel: DVec3,
    /// Orientation: turns the axes of the thing into the axes of the frame.
    pub rot: DQuat,
}

/// How a frame sits in the frame of its star: `star = offset + turn * local`, the frame moving
/// at `vel` and turning at `omega`, both in star axes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rigid {
    /// Origin of the frame in the star frame.
    pub offset: DVec3,
    /// Rotation from frame axes to star axes.
    pub turn: DQuat,
    /// Velocity of the origin in the star frame.
    pub vel: DVec3,
    /// Angular velocity of the frame in star axes.
    pub omega: DVec3,
}

/// Returns the velocity of a body at `time` seconds in the frame of its star, the way the
/// ship flight measures it.
pub fn velocity(spec: &PlanetSpec, time: f64) -> DVec3 {
    spec.position(time + 0.5) - spec.position(time - 0.5)
}

impl Rigid {
    /// The frame of the star itself.
    pub const STILL: Self = Self {
        offset: DVec3::ZERO,
        turn: DQuat::IDENTITY,
        vel: DVec3::ZERO,
        omega: DVec3::ZERO,
    };

    /// Returns the turning frame of the body `spec` at `time`, or the frame around it when
    /// `turning` is false.
    pub fn body(spec: &PlanetSpec, time: f64, turning: bool) -> Self {
        let around = Self {
            offset: spec.position(time),
            vel: velocity(spec, time),
            ..Self::STILL
        };
        match turning {
            true => Self {
                turn: spin(spec.day_seconds, time),
                omega: DVec3::Y * (TAU / spec.day_seconds),
                ..around
            },
            false => around,
        }
    }

    /// Returns a pose of this frame in the star frame.
    pub fn out(&self, pose: &Pose) -> Pose {
        let arm = self.turn * pose.pos;
        Pose {
            pos: self.offset + arm,
            vel: self.vel + self.turn * pose.vel + self.omega.cross(arm),
            rot: (self.turn * pose.rot).normalize(),
        }
    }

    /// Returns a pose of the star frame in this frame.
    pub fn back(&self, pose: &Pose) -> Pose {
        let (arm, undo) = (pose.pos - self.offset, self.turn.inverse());
        Pose {
            pos: undo * arm,
            vel: undo * (pose.vel - self.vel - self.omega.cross(arm)),
            rot: (undo * pose.rot).normalize(),
        }
    }
}
