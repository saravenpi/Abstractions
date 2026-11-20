//! Stagger: the push of a blow on the robot, how long it takes to recover and whether it loses its
//! footing, and the lean and the fall that show it.

use bevy::math::Vec2;
use bevy::prelude::*;

use super::super::pose::{Core, Pose};
use super::tune::{
    ABSORB_MAX, ARM_KG, BRACE, CAPACITY, DOWN, FALL_AT, KNEES, LEAN_MAX, LIE, LYING, RECOVER_S,
    RECOVER_SPAN, RISE, SHOVE_MAX, STAGGER_MIN,
};

/// The stagger of a body, on its root: the pose is leaned and, past the footing, laid down and raised.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Reel {
    /// Unit direction of the push in the frame of the body: `x` to the right, `y` ahead.
    pub push: Vec2,
    /// Lean at the deepest, in radians.
    pub lean: f32,
    /// Seconds since the blow.
    pub age: f32,
    /// Seconds it lasts in all.
    pub time: f32,
    /// True when the body lost its footing.
    pub fall: bool,
}

/// Returns the speed a blow of `momentum` kg·m/s gives a body of `mass` kilograms, in metres per
/// second, at most [`SHOVE_MAX`].
pub fn impulse(momentum: f32, mass: f32) -> f32 {
    (momentum.max(0.0) / mass.max(f32::EPSILON)).min(SHOVE_MAX)
}

/// Returns the share of a blow a tool of `mass` kilograms in a guard takes: its mass over its mass
/// and the arm's, at most [`ABSORB_MAX`].
pub fn absorb(mass: f32) -> f32 {
    (mass / (mass + ARM_KG)).min(ABSORB_MAX)
}

/// Returns the impulse a stance leaves of `impulse` when a guard that takes `share` of a blow meets
/// it, `None` for no guard.
pub fn guarded(impulse: f32, share: Option<f32>) -> f32 {
    impulse * (1.0 - share.unwrap_or(0.0))
}

/// Returns how much impulse a stance absorbs, in metres per second: [`CAPACITY`], doubled by a full
/// crouch and widened by [`BRACE`] when `braced`.
pub fn capacity(crouch: f32, braced: bool) -> f32 {
    CAPACITY * (1.0 + crouch.clamp(0.0, 1.0)) * if braced { 1.0 + BRACE } else { 1.0 }
}

/// Returns the stagger an `impulse` makes against a stance of `capacity`: nothing under
/// [`STAGGER_MIN`], else the push along `push`, with a recovery that grows with the impulse over the
/// capacity, and a fall once the impulse passes [`FALL_AT`] scaled by the capacity.
pub fn reel(impulse: f32, capacity: f32, push: Vec2) -> Option<Reel> {
    if impulse < STAGGER_MIN {
        return None;
    }
    let ratio = impulse / capacity.max(f32::EPSILON);
    let fall = impulse > FALL_AT * capacity / CAPACITY;
    let recover = RECOVER_S * ratio.clamp(RECOVER_SPAN[0], RECOVER_SPAN[1]);
    Some(Reel {
        push: push.normalize_or_zero(),
        lean: LEAN_MAX * (impulse / FALL_AT).clamp(0.2, 1.0),
        age: 0.0,
        time: if fall {
            recover + DOWN[0] + LIE + DOWN[1]
        } else {
            recover
        },
        fall,
    })
}

impl Reel {
    /// Returns how deep the body is in its stagger, 0 to 1: it builds over [`RISE`] and eases out to
    /// nothing; a fall goes down over [`DOWN`], lies, and gets up.
    pub fn depth(&self) -> f32 {
        let t = self.age;
        if self.fall {
            let up = (t / DOWN[0]).min(1.0);
            let rise = ((self.time - t) / DOWN[1]).clamp(0.0, 1.0);
            return up.min(rise);
        }
        let build = (t / RISE).min(1.0);
        let left =
            (1.0 - (t - RISE).max(0.0) / (self.time - RISE).max(f32::EPSILON)).clamp(0.0, 1.0);
        build * left * left
    }

    /// Returns `pose` leaned along the push by the depth of the stagger: the body pitches with the push
    /// ahead or behind it and rolls with the push to a side, the knees give, and a fall lays it down.
    pub fn apply(&self, pose: Pose) -> Pose {
        let d = self.depth();
        let (pitch, roll) = if self.fall {
            (LYING * self.push.y.signum(), LEAN_MAX * self.push.x)
        } else {
            (self.lean * self.push.y, self.lean * self.push.x)
        };
        pose.with(Core::Pitch, pose.core(Core::Pitch) + pitch * d)
            .with(Core::Roll, pose.core(Core::Roll) + roll * d)
            .with(Core::Lift, pose.core(Core::Lift) - KNEES * d)
            .bounded()
    }
}
