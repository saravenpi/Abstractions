//! A body brought down onto the ground by the real gear: the hit, the touchdown the gear makes of
//! it and what follows, filled, dressed and posed frame by frame the way the game does it.

use bevy::prelude::*;

use super::FRAME;
use crate::avatar::animate::Anim;
use crate::avatar::blend::Weights;
use crate::avatar::dress::dress;
use crate::avatar::fill::{Memory, observe};
use crate::avatar::pose::{Core, Pose};
use crate::avatar::{AvatarPose, Mode};
use crate::gear::{FlightState, GearInput, Mode as Flight, Servo, slid, step};
use crate::player::Motion;

/// Free height under the body in the air, in meters: it is about to touch down.
const CLEARANCE: f32 = 0.3;

/// Frames the body is posed in the air before it hits, to settle.
pub const SETTLE: usize = 90;

/// A body touching down, and everything that poses it.
pub struct Touchdown {
    /// The gear.
    pub state: FlightState,
    /// What the gear is told.
    input: GearInput,
    /// What the filler remembers.
    memory: Memory,
    /// The pose input.
    pub pose: AvatarPose,
    /// The animation.
    pub anim: Anim,
    /// The pose of every frame so far, the turn of the roll taken out, with the turn itself.
    pub frames: Vec<(Pose, Vec2)>,
}

/// Returns `pose` with the whole-body turn `turn` of a roll taken out of its lean and roll.
pub fn unturned(pose: &Pose, turn: Vec2) -> Pose {
    pose.with(Core::Pitch, pose.core(Core::Pitch) - turn.x)
        .with(Core::Roll, pose.core(Core::Roll) - turn.y)
}

impl Touchdown {
    /// Returns a body in the air in the gear mode `flight`, heading `yaw`, moving at `velocity` in
    /// the upright frame and lying as `prone` as the gear says, its pose settled in `mode`.
    pub fn new(flight: Flight, velocity: Vec3, (yaw, prone): (f32, f32), mode: Mode) -> Self {
        let winged = flight.winged();
        let state = FlightState {
            mode: flight,
            velocity,
            wings: f32::from(u8::from(winged)),
            prone,
            servo: Servo { yaw, ..default() },
            ..FlightState::default()
        };
        let mut body = Self {
            state,
            input: GearInput {
                deploy: winged,
                ..default()
            },
            memory: Memory::default(),
            pose: AvatarPose::default(),
            anim: Anim {
                weights: Weights::only(mode),
                ..default()
            },
            frames: Vec::new(),
        };
        for _ in 0..SETTLE {
            body.frame(velocity, false);
        }
        body
    }

    /// Runs one frame: the filler reads the controller moving at `moved`, on the ground or not,
    /// the dresser the gear, and the body is posed.
    fn frame(&mut self, moved: Vec3, grounded: bool) {
        let motion = Motion {
            velocity: moved,
            yaw: self.state.servo.yaw,
            grounded,
            ..default()
        };
        observe(&mut self.pose, &mut self.memory, &motion, (false, FRAME));
        dress(&mut self.pose, &self.state, Vec2::ZERO, false);
        self.pose.clearance = if grounded { 0.0 } else { CLEARANCE };
        let pose = self.anim.step(&self.pose, FRAME);
        self.frames
            .push((unturned(&pose, self.anim.clock.turn), self.anim.clock.turn));
    }

    /// Hits the ground: the physics takes the part of the velocity that goes into it.
    ///
    /// With the gear off the controller still moved the body at its fall speed this frame.
    pub fn hit(&mut self) {
        let before = self.state.velocity;
        self.state = slid(&self.state, before.with_y(0.0));
        let moved = match self.state.mode.drives() {
            true => self.state.velocity,
            false => before,
        };
        self.frame(moved, false);
    }

    /// Runs `frames` frames on the ground, the gear stepping and the physics holding it there.
    /// With the gear off the controller stops the body, so it stands and no stride swings the
    /// legs: what is left to measure is the touchdown.
    pub fn ground(&mut self, frames: usize) {
        self.input = GearInput {
            grounded: true,
            ..default()
        };
        for _ in 0..frames {
            self.state = step(&self.state, &self.input, FRAME);
            if !self.state.mode.drives() {
                self.state.velocity = Vec3::ZERO;
            }
            self.state = slid(&self.state, self.state.velocity.with_y(0.0));
            self.frame(self.state.velocity, true);
        }
    }
}
