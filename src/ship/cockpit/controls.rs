//! The controls of the cockpit: where the stick, the throttle lever, the pedals, the gear
//! lever and the warp switch stand for what the pilot asks of the ship, and where the hands
//! hold them. Every angle is a pure function of its input and stays in its range.

use bevy::prelude::*;

use super::tune::{FOLLOW, LEVER, LEVER_LENGTH, LEVER_SWING, STICK, STICK_LENGTH, STICK_REACH};

/// Where the controls of a ship are: what its pilot asks of it, or where its parts stand.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Levers {
    /// The stick, from -1 to 1 around the body axes: pitch up, yaw left, roll left.
    pub stick: Vec3,
    /// The throttle lever, from 0 (idle) to 1.
    pub throttle: f32,
    /// The gear lever, from 0 (gear up) to 1 (gear down).
    pub gear: f32,
    /// The warp switch: 0 with the drive off, 0.5 while it spools, 1 when it is engaged.
    pub warp: f32,
}

/// Returns the turn of a lever that swings `swing` radians either way of upright, back at
/// `share` 0 and forward at 1. A share out of range is held to it.
pub(super) fn throw(swing: f32, share: f32) -> Quat {
    Quat::from_rotation_x(swing * (1.0 - 2.0 * share.clamp(0.0, 1.0)))
}

impl Levers {
    /// Returns the controls `dt` seconds later, on their way to `asked`.
    pub fn eased(self, asked: Levers, dt: f32) -> Levers {
        let share = 1.0 - (-FOLLOW * dt).exp();
        let ease = |now: f32, to: f32| now + (to - now) * share;
        Levers {
            stick: self.stick + (asked.stick - self.stick) * share,
            throttle: ease(self.throttle, asked.throttle),
            gear: ease(self.gear, asked.gear),
            warp: ease(self.warp, asked.warp),
        }
    }

    /// Returns the turn of the stick in its hinge: pulled back for pitch up, tilted left for
    /// a roll to the left, twisted left for a yaw to the left.
    pub fn stick_turn(&self) -> Quat {
        let held = self.stick.clamp(Vec3::NEG_ONE, Vec3::ONE);
        Quat::from_rotation_x(held.x * STICK_REACH.0)
            * Quat::from_rotation_z(held.z * STICK_REACH.0)
            * Quat::from_rotation_y(held.y * STICK_REACH.1)
    }

    /// Returns the turn of the throttle lever in its hinge: back at idle, forward at full.
    pub fn lever_turn(&self) -> Quat {
        throw(LEVER_SWING, self.throttle)
    }

    /// Returns where the hands of the pilot hold the controls, in hull axes: the left hand on
    /// the throttle, the right hand on the stick.
    pub fn grips(&self) -> [Vec3; 2] {
        let lever = Vec3::Y * (LEVER_LENGTH.0 - LEVER_LENGTH.1);
        let stick = Vec3::Y * (STICK_LENGTH.0 - STICK_LENGTH.1);
        [
            LEVER + self.lever_turn() * lever,
            STICK + self.stick_turn() * stick,
        ]
    }
}
