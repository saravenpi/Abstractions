//! The ship state: where it is, how it moves and what its drive does, in an explicit frame.

use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::warp::Warp;
use crate::space::Frame;
use crate::universe::{BodyId, GalaxyId, StarId};

/// The player's ship: a rigid body simulated in 64 bits.
///
/// Body axes follow Bevy: the nose points along -Z, the roof along +Y and the right wing along +X.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Ship {
    /// Frame `pos`, `vel` and `rot` are expressed in.
    pub frame: Frame,
    /// Position of the hull centre in metres.
    pub pos: DVec3,
    /// Velocity in metres per second, relative to the frame.
    pub vel: DVec3,
    /// Orientation: turns body axes into frame axes.
    pub rot: DQuat,
    /// Angular velocity in radians per second around the body axes: pitch up, yaw left, roll left.
    pub spin: DVec3,
    /// Main engine lever from -0.5 (reverse) to 1.
    pub throttle: f64,
    /// True while the ship rests on its gear or floats at rest. A landed ship is not integrated.
    pub landed: bool,
    /// Damage taken from hard landings, from 0 (none) to 1.
    pub damage: f64,
    /// Whether the flight assist is on.
    pub assist: bool,
    /// State of the warp drive.
    pub warp: Warp,
    /// Seconds left during which the gear damps weakly, after a landing above the safe speed.
    pub bounce: f64,
    /// Whether any contact point touched the ground on the last step.
    pub touching: bool,
    /// Seconds the ship has been slow and held by the ground or the sea, before it comes to rest.
    pub calm: f64,
}

impl Ship {
    /// Returns a ship at rest at `pos` with orientation `rot` in `frame`, assist on, engines idle.
    pub fn at_rest(frame: Frame, pos: DVec3, rot: DQuat) -> Self {
        Self {
            frame,
            pos,
            vel: DVec3::ZERO,
            rot,
            spin: DVec3::ZERO,
            throttle: 0.0,
            landed: false,
            damage: 0.0,
            assist: true,
            warp: Warp::Off,
            bounce: 0.0,
            touching: false,
            calm: 0.0,
        }
    }

    /// Returns the unit direction of the nose in frame axes.
    pub fn forward(&self) -> DVec3 {
        self.rot * DVec3::NEG_Z
    }

    /// Returns the plain record of the ship for a save file.
    pub fn state(&self) -> ShipState {
        ShipState {
            frame: self.frame.into(),
            pos: self.pos.to_array(),
            vel: self.vel.to_array(),
            rot: self.rot.to_array(),
            landed: self.landed,
        }
    }

    /// Rebuilds a ship from a saved record: engines idle, drive off, assist on, no spin.
    pub fn from_state(state: &ShipState) -> Self {
        let rot = DQuat::from_array(state.rot);
        let rot = if rot.length_squared() > 1e-12 {
            rot.normalize()
        } else {
            DQuat::IDENTITY
        };
        Self {
            vel: DVec3::from_array(state.vel),
            landed: state.landed,
            touching: state.landed,
            ..Self::at_rest(state.frame.into(), DVec3::from_array(state.pos), rot)
        }
    }
}

/// The frame of a saved ship: [`Frame`] in a form a save file can hold.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SavedFrame {
    /// Fixed to a planet and turning with it.
    Surface(BodyId),
    /// Inertial, origin at a star.
    System(StarId),
    /// Inertial, origin at a galaxy centre.
    Galaxy(GalaxyId),
}

impl From<Frame> for SavedFrame {
    /// Returns the saved form of a frame.
    fn from(frame: Frame) -> Self {
        match frame {
            Frame::Surface(body) => Self::Surface(body),
            Frame::System(star) => Self::System(star),
            Frame::Galaxy(galaxy) => Self::Galaxy(galaxy),
        }
    }
}

impl From<SavedFrame> for Frame {
    /// Returns the frame a saved form stands for.
    fn from(frame: SavedFrame) -> Self {
        match frame {
            SavedFrame::Surface(body) => Self::Surface(body),
            SavedFrame::System(star) => Self::System(star),
            SavedFrame::Galaxy(galaxy) => Self::Galaxy(galaxy),
        }
    }
}

/// What a save file keeps of the ship. Plain data: metres, metres per second, a unit quaternion.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct ShipState {
    /// Frame the other fields are expressed in.
    pub frame: SavedFrame,
    /// Position of the hull centre.
    pub pos: [f64; 3],
    /// Velocity relative to the frame.
    pub vel: [f64; 3],
    /// Orientation as `[x, y, z, w]`.
    pub rot: [f64; 4],
    /// Whether the ship rests on the ground or on the sea.
    pub landed: bool,
}
