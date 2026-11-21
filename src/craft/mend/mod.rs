//! Mending the robot. At the home base every part mends itself for free. In the field the robot
//! patches the worst part with what it carries, through the same grammar the tools use: a plate
//! needs a hard material, a joint needs a binding.

mod base;
mod field;
#[cfg(test)]
mod tests;
pub mod tune;

pub use field::{patch, supply, worst};

use bevy::prelude::*;

use crate::home::Base;
use crate::screens::{in_control, playing};

/// Registers the bay of the base and the mend key.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            base::tend.run_if(resource_exists::<Base>).run_if(playing),
            field::press.run_if(in_control),
        ),
    );
}
