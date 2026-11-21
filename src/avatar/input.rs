//! The input of a body: everything it needs to pose itself, filled once a frame.

use bevy::prelude::*;

use super::Mode;

/// Everything a body needs to pose itself, on its root. Angles are radians, speeds meters per
/// second. The filler and the dresser write the one of the player at this machine; whatever
/// writes the one of another body makes that body move the same way.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct AvatarPose {
    /// What the body is doing.
    pub mode: Mode,
    /// Horizontal speed.
    pub speed: f32,
    /// Speed along the local up, negative when falling.
    pub vertical_speed: f32,
    /// Distance walked so far in meters, ever growing. The leg cycle follows it, so feet do not slide.
    pub stride_distance: f32,
    /// Bank into a turn while flying: positive rolls the right side down.
    pub bank: f32,
    /// Pitch of the flight path: positive climbs.
    pub pitch: f32,
    /// Thrust of the boot boosters, from 0 to 1.
    pub thrust: f32,
    /// How far the wingsuit is asked to deploy, from 0 (folded) to 1.
    pub wings: f32,
    /// Speed through the air, which makes the fabric flutter.
    pub airspeed: f32,
    /// Where the head looks relative to the body: `x` is the yaw to the left, `y` the pitch up.
    pub look: Vec2,
    /// How hard the last landing was, from 0 (soft) to 1 (hard). Read when the mode turns to `Land`.
    pub landed_hard: f32,
    /// Speed along the local up the body lost in its last touch-down, in meters per second.
    pub impact: f32,
    /// Velocity the body had when the gear last hit a surface, in meters per second in the frame
    /// of the body: +X right, +Y up, -Z ahead.
    pub struck: Vec3,
    /// Speed the body lost into that surface, in meters per second.
    pub lost: f32,
    /// True for a moment after any touch-down, at a run too: the knees take the landing in.
    pub absorb: bool,
    /// How fast the ground speed grows, eased, in meters per second squared. Negative brakes.
    pub gain: f32,
    /// How fast the heading turns to the left on the ground, eased, in radians per second.
    pub turn: f32,
    /// Free height under the body down to the ground, in meters. Zero on the ground.
    pub clearance: f32,
    /// How prone the flight gear lays the body, from 0 (upright) to 1 (flat along its nose).
    pub prone: f32,
    /// How far the body is crouched, from 0 (standing) to 1 (fully down).
    pub crouch: f32,
    /// Felt load in g while the gear carries the body: 1 in steady flight, more in a pull-up.
    pub load: f32,
    /// Orientation the flight gear holds the body at, in the frame of the player entity: +Y
    /// the head, -Z the chest. The aero model flies this very attitude, and the winged poses
    /// lie at it exactly, blended with every other pose by the shortest rotation. None where
    /// nothing measures it, as for the body of another player: the winged poses then lie
    /// along `pitch` and `bank`.
    pub attitude: Option<Quat>,
}
