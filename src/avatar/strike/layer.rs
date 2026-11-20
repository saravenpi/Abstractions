//! The strike layer: what a strike lays over the blended pose of the body, the chest turned and the
//! arms on the grip.

use bevy::prelude::*;

use super::super::pose::{Core, Limb, Pose};
use super::act::{Strike, Timing, key};
use super::held::Held;
use super::ik::reach;
use super::key::Key;
use super::path::Path;
use super::tune::SECOND_HAND;
use crate::craft::build::Hands;

/// The joints of an arm the layer sets, in the order [`reach`] gives them.
const ARM: [Limb; 3] = [Limb::Shoulder, Limb::ArmOut, Limb::Elbow];

/// What a strike lays over the pose of a body, on its root. A weight of 0 leaves the pose alone.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Layer {
    /// How much of the layer is in the pose, 0 to 1.
    pub weight: f32,
    /// Chest turn and bend added to the pose, in radians.
    pub trunk: [f32; 2],
    /// Shoulder, lift and elbow of each arm that holds the grip, left then right.
    pub arms: [Option<[f32; 3]>; 2],
}

impl Layer {
    /// Returns the layer that poses `key` held in `hands`: the right hand on the grip, and the left
    /// one further down the shaft when both hold it.
    pub fn of(key: &Key, hands: Hands) -> Self {
        Self::held(key, hands, [true; 2])
    }

    /// Returns the layer that poses `key` held in `hands` by the arms that `usable` leaves, left then
    /// right: a tool of two hands is held by the one arm that is left, and a body with no arm left
    /// holds nothing.
    pub fn held(key: &Key, hands: Hands, usable: [bool; 2]) -> Self {
        let second = || reach(key.grip - key.axis * SECOND_HAND, 0);
        let arms = match usable {
            [true, true] => [(hands == Hands::Two).then(second), Some(reach(key.grip, 1))],
            [_, true] => [None, Some(reach(key.grip, 1))],
            [true, false] => [Some(reach(key.grip, 0)), None],
            [false, false] => [None, None],
        };
        Self {
            weight: 1.0,
            trunk: [key.twist, key.spine],
            arms,
        }
    }

    /// Returns `pose` with this layer laid over it: the chest turn and bend added, the arms that
    /// hold the grip blended to their reach, every joint kept inside its limits.
    pub fn apply(&self, pose: Pose) -> Pose {
        if self.weight <= 0.0 {
            return pose;
        }
        let w = self.weight.min(1.0);
        let mut out = pose
            .with(Core::Twist, pose.core(Core::Twist) + self.trunk[0] * w)
            .with(Core::Spine, pose.core(Core::Spine) + self.trunk[1] * w);
        for (side, arm) in self.arms.iter().enumerate() {
            let Some(arm) = arm else { continue };
            for (joint, value) in ARM.into_iter().zip(arm) {
                let base = out.sides[side][joint as usize];
                out.sides[side][joint as usize] = base + (value - base) * w;
            }
        }
        out.bounded()
    }
}

/// Returns the key `strike` poses with `held` now.
pub fn posed(strike: &Strike, held: &Held) -> Key {
    let path = Path::of(held.style, strike.dir);
    key(
        strike.act,
        &path,
        &Timing::of(held, &path).felt(&strike.hurt),
        strike.dir,
    )
}

/// Lays the strike of every body over its pose: the key of its act with the tool out, nothing with
/// the tool stowed or gone.
pub fn lay(mut bodies: Query<(&mut Layer, &Strike, Option<&Held>)>) {
    for (mut layer, strike, held) in &mut bodies {
        let next = match held.filter(|held| held.out) {
            Some(held) => Layer::held(&posed(strike, held), held.build.hands, strike.hurt.usable),
            None => Layer::default(),
        };
        layer.set_if_neq(next);
    }
}
