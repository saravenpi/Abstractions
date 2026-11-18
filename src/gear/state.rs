//! The state of the flight gear between two steps. Plain copyable data, no heap.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::mode::Mode;
use crate::render::smoothstep as smooth;

/// Inner state of the attitude response.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Servo {
    /// Rates of pitch, bank, proneness and pull, per second.
    pub rates: Vec4,
    /// Angle in radians the nose is pulled toward the dorsal side, on top of the pitch: the
    /// trim and the stick. It acts in the banked plane, which is what tightens a turn.
    pub pull: f32,
    /// Heading of the nose in radians around the local up.
    pub yaw: f32,
    /// Seconds spent in the current mode.
    pub clock: f32,
    /// Speed lost into the last surface hit, in metres per second, until a touchdown reads it.
    pub impact: f32,
}

/// The flight gear at one instant.
///
/// Vectors and the attitude are in the upright frame of the body: +Y is the local up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlightState {
    /// What the gear is doing.
    pub mode: Mode,
    /// Velocity in metres per second. While the mode drives, move the body by it.
    pub velocity: Vec3,
    /// Orientation of the body model: +Y the head, -Z the chest. Upright on foot, prone in flight.
    pub attitude: Quat,
    /// Wing spread progress from 0 (folded) to 1, linear in time.
    pub wings: f32,
    /// Booster spool progress from 0 (off) to 1, linear in time.
    pub thrust: f32,
    /// Speed through the air in metres per second.
    pub airspeed: f32,
    /// Angle of attack in radians: how far the nose points above the flight path.
    pub aoa: f32,
    /// Bank in radians, positive with the right wing down.
    pub bank: f32,
    /// Angle of the flight frame above the horizon in radians, before the pull. With the wings
    /// folded it follows the fall, so that spreading them never meets the air flat on.
    pub pitch: f32,
    /// How prone the body lies, from 0 (upright) to 1 (flat along the nose).
    pub prone: f32,
    /// Felt load in g: the push of air and boosters over the local weight.
    pub load: f32,
    /// Inner state of the attitude response.
    pub servo: Servo,
}

impl Default for FlightState {
    /// At rest, upright, gear off.
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            velocity: Vec3::ZERO,
            attitude: Quat::IDENTITY,
            wings: 0.0,
            thrust: 0.0,
            airspeed: 0.0,
            aoa: 0.0,
            bank: 0.0,
            pitch: 0.0,
            prone: 0.0,
            load: 0.0,
            servo: Servo::default(),
        }
    }
}

impl FlightState {
    /// Returns how much of the wing is spread, from 0 to 1: the progress eased so that the
    /// motion starts and ends at rest. The lift, the drag and the drawn wing all read this one
    /// value, so the body is carried by exactly the wing the eye sees.
    pub fn spread(&self) -> f32 {
        smooth(0.0, 1.0, self.wings)
    }

    /// Returns how hard the boosters push, eased from the spool progress.
    pub fn power(&self) -> f32 {
        smooth(0.0, 1.0, self.thrust)
    }

    /// Returns the flight frame: -Z the nose, +Y the dorsal side, +X the right wing.
    ///
    /// Heading, then pitch, then bank, then the pull inside the banked plane.
    pub fn frame(&self) -> Quat {
        let banked = Quat::from_euler(EulerRot::YXZ, self.servo.yaw, self.pitch, -self.bank);
        banked * Quat::from_rotation_x(self.servo.pull)
    }

    /// Returns the unit direction of the nose.
    pub fn nose(&self) -> Vec3 {
        self.frame() * Vec3::NEG_Z
    }

    /// Returns the orientation of the body model: upright on its heading while not prone, flat
    /// along the flight frame when fully prone, turning from one to the other in between.
    pub fn pose(&self) -> Quat {
        let upright = Quat::from_rotation_y(self.servo.yaw);
        let flat = self.frame() * Quat::from_rotation_x(-FRAC_PI_2);
        upright.slerp(flat, self.prone).normalize()
    }
}
