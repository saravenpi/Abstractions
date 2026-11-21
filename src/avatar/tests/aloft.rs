//! A body flown by the real flight gear and posed by the real avatar, dressed as the game
//! dresses it: the attitude of the gear in the frame of a player entity turned to its nose.

use bevy::prelude::*;

use super::FRAME;
use crate::avatar::animate::Anim;
use crate::avatar::blend::Weights;
use crate::avatar::dress::{attitude, dress};
use crate::avatar::rig::{Seg, globals};
use crate::avatar::{AvatarPose, Mode};
use crate::gear::{FlightState, GearInput, Mode as Flight, step};

/// Largest turn of the whole body in one frame at sixty frames a second, in radians (7 degrees):
/// a quarter turn takes at least a fifth of a second.
pub const SNAP: f32 = 0.12;

/// A body in the air, its gear and its animation.
pub struct Aloft {
    /// The flight gear.
    pub state: FlightState,
    /// The animation of the body.
    pub anim: Anim,
    /// The attitude the dresser keeps from the last moment in the air.
    held: Quat,
    /// The turn of the pelvis in the upright frame on the last frame.
    pub body: Quat,
}

impl Aloft {
    /// Returns a body in the gear `state`, posed in `mode` from the start. Run it a little
    /// before measuring: its first frame turns from nowhere.
    pub fn new(state: FlightState, mode: Mode) -> Self {
        let anim = Anim {
            weights: Weights::only(mode),
            ..Anim::default()
        };
        Self {
            state,
            anim,
            held: Quat::IDENTITY,
            body: Quat::IDENTITY,
        }
    }

    /// Returns a body falling at `velocity` with no gear in use, as the controller hands it to
    /// the gear, looking ahead a little down.
    pub fn falling(velocity: Vec3) -> Self {
        let state = FlightState {
            velocity,
            ..FlightState::default()
        };
        let mut aloft = Self::new(state, Mode::Fall);
        aloft.run(&folded(), 0.5);
        aloft
    }

    /// Returns a body settled in a hands-off glide at `velocity`.
    pub fn gliding(velocity: Vec3) -> Self {
        let state = FlightState {
            mode: Flight::Glide,
            velocity,
            wings: 1.0,
            prone: 1.0,
            ..FlightState::default()
        };
        let mut aloft = Self::new(state, Mode::Glide);
        aloft.run(&spread(), 3.0);
        aloft
    }

    /// Steps the gear and the body one frame under `input`, dressed as the game dresses the
    /// body of the player at this machine, and returns how far the pelvis turned in the
    /// upright frame, in radians.
    pub fn tick(&mut self, input: &GearInput) -> f32 {
        self.state = step(&self.state, input, FRAME);
        let v = self.state.velocity;
        let mut pose = AvatarPose {
            mode: Mode::Fall,
            speed: v.with_y(0.0).length(),
            vertical_speed: v.y,
            airspeed: v.length(),
            clearance: 500.0,
            ..AvatarPose::default()
        };
        let yaw = self.state.servo.yaw;
        dress(&mut pose, &self.state, Vec2::ZERO, false);
        pose.attitude = Some(attitude(&self.state, yaw, &mut self.held));
        let posed = self.anim.step(&pose, FRAME);
        let body = Quat::from_rotation_y(yaw) * globals(&posed)[Seg::Pelvis as usize].rotation;
        let turn = body.angle_between(self.body);
        self.body = body;
        turn
    }

    /// Runs `seconds` under `input` and returns the largest turn of the pelvis in one frame.
    pub fn run(&mut self, input: &GearInput, seconds: f32) -> f32 {
        let frames = (seconds / FRAME).round() as u32;
        (0..frames).map(|_| self.tick(input)).fold(0.0, f32::max)
    }
}

/// Returns the input of a body falling with the wings folded, the view a little down.
pub fn folded() -> GearInput {
    GearInput {
        aim: Some(Vec2::new(0.0, -0.2)),
        ..GearInput::default()
    }
}

/// Returns the input of the wings asked open and flown hands off.
pub fn spread() -> GearInput {
    GearInput {
        deploy: true,
        ..GearInput::default()
    }
}
