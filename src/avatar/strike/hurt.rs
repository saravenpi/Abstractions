//! Hurt arms and a hurt torso: what they leave of a swing, a throw and a grip.
//!
//! The arms set the speed of the swing and the throw and the steadiness of the aim, the torso the
//! recovery after a swing. A broken arm lets go of the tool, and a tool that wants two hands is
//! then held in one, which makes its wind-up longer.

use bevy::prelude::*;

use super::act::{Strike, Timing};
use super::held::Held;
use super::input::Robot;
use super::tune::ONE_HAND;
use crate::condition::{Condition, luck, tune::SHAKE};
use crate::craft::build::Hands;

/// What the body of a robot leaves of its strikes. A whole body leaves everything.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hurt {
    /// Share of the swing and throw speed the arms leave, 0 to 1.
    pub swing: f32,
    /// Share of the recovery speed the torso leaves, 0 to 1.
    pub recover: f32,
    /// How steady the aim is, from 0 (shaking) to 1.
    pub steady: f32,
    /// Which arms can hold the tool, left then right.
    pub usable: [bool; 2],
    /// True when the tool takes two hands and only one is left to hold it.
    pub short: bool,
}

impl Default for Hurt {
    /// A whole body.
    fn default() -> Self {
        Self {
            swing: 1.0,
            recover: 1.0,
            steady: 1.0,
            usable: [true; 2],
            short: false,
        }
    }
}

impl Hurt {
    /// Returns what `condition` leaves of the strikes with a tool held in `hands`: the two arms
    /// share a two-handed swing, the arm that is left carries a one-handed one, none carries nothing.
    pub fn of(condition: &Condition, hands: Hands) -> Self {
        let usable = [!condition.broken(0), !condition.broken(1)];
        let two = hands == Hands::Two;
        let power = match usable {
            [true, true] if two => (condition.arm(0) + condition.arm(1)) / 2.0,
            [_, true] => condition.arm(1),
            [true, false] => condition.arm(0),
            [false, false] => 0.0,
        };
        Self {
            swing: power,
            recover: condition.recovery(),
            steady: power,
            usable,
            short: two && usable != [true, true],
        }
    }

    /// Returns the factor the wind-up is stretched by: a tool made for two hands is slow in one.
    pub fn grip(&self) -> f32 {
        match self.short {
            true => ONE_HAND,
            false => 1.0,
        }
    }
}

impl Timing {
    /// Returns this timing as a hurt body keeps it: slower swings and wind-ups for weak arms, a
    /// longer wind-up in one hand for a tool of two, a slower recovery for a weak torso.
    pub fn felt(self, hurt: &Hurt) -> Self {
        let arm = hurt.swing.max(f32::EPSILON);
        Self {
            windup: self.windup * hurt.grip() / arm,
            swing: self.swing / arm,
            recover: self.recover / hurt.recover.max(f32::EPSILON),
        }
    }
}

/// Returns `aim` turned a little in a direction the roll `key` picks, by up to [`SHAKE`] radians
/// for an arm that cannot hold it steady.
pub fn steadied(aim: Vec3, steady: f32, key: u32) -> Vec3 {
    let turn = SHAKE * (1.0 - steady.clamp(0.0, 1.0));
    let (yaw, pitch) = (
        (luck(key) - 0.5) * 2.0 * turn,
        (luck(key ^ 0x9e37) - 0.5) * 2.0 * turn,
    );
    Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0) * aim
}

/// Writes what the condition of the robot leaves of its strikes into the strike of its body, so the
/// systems of the hands read it with the tool.
pub fn feel(
    robot: Robot,
    condition: Option<Res<Condition>>,
    mut bodies: Query<(&mut Strike, &Held)>,
) {
    let Some(body) = robot.body() else { return };
    let Ok((mut strike, held)) = bodies.get_mut(body) else {
        return;
    };
    let hurt = condition.map_or(Hurt::default(), |c| Hurt::of(&c, held.build.hands));
    if strike.hurt != hurt {
        strike.hurt = hurt;
    }
}
