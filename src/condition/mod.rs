//! The condition of the robot: how whole each of its parts is, and what that does to the systems
//! that drive them.
//!
//! The model is pure data in the base layer, so the controller, the gear, the pack, the hands and
//! the companion read it without importing the body. Whatever hurts the robot lowers a part with
//! [`Condition::hurt`]; the body draws the harm and the HUD lists it.

mod effects;
mod gear;
mod luck;
mod part;
mod shield;
mod summary;
pub mod tune;

pub use luck::luck;
pub use part::{PARTS, Part};
pub use shield::Shield;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

use tune::{BLUNT, CUT, PLATE};

/// How whole each part of the robot is, from 1 (whole) to 0 (destroyed).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Condition {
    /// The health of every part, in the order of [`Part::ALL`].
    whole: [f32; PARTS],
}

/// The condition of the body of another player, on its puppet: the walk of the puppet limps by it
/// as the walk of the local robot limps by the [`Condition`] resource.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Wounded(pub Condition);

impl Default for Condition {
    /// A robot with every part whole.
    fn default() -> Self {
        Self {
            whole: [1.0; PARTS],
        }
    }
}

impl Condition {
    /// Returns the health of `part`.
    pub fn of(&self, part: Part) -> f32 {
        self.whole[part as usize]
    }

    /// Takes the harm of a blow of `joules` from `part`: a cut goes through its plate, a blunt blow
    /// is spread by it. Returns the share of health lost.
    pub fn hurt(&mut self, part: Part, joules: f32, cut: bool) -> f32 {
        let reach = if cut { CUT } else { BLUNT };
        let lost = (joules.max(0.0) * reach / PLATE[part as usize]).min(self.of(part));
        self.whole[part as usize] -= lost;
        lost
    }

    /// Gives `part` back `amount` of health, up to `cap`. Returns the share given.
    pub fn mend(&mut self, part: Part, amount: f32, cap: f32) -> f32 {
        let gain = amount.min(cap - self.of(part)).max(0.0);
        self.whole[part as usize] += gain;
        gain
    }
}

/// Registers the condition and energy shield of the robot.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Condition>()
        .init_resource::<Shield>()
        .add_systems(Update, shield::recharge);
}
