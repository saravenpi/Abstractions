//! Attacks on and by animals, and what is left of an animal that died.
//!
//! The robot's blows hurt an animal ([`harm`]), push and may break the part they struck ([`limbs`]),
//! and a kill makes it fall ([`slump`]) and stay as a carcass ([`slay`]) that scavengers come to and
//! that decays into bones. The robot cuts parts from the carcass ([`carve`]). Animals that fight or
//! hunt the robot pick an attack from their body ([`kinds`]), wind up with a telegraph that grows with
//! their size ([`timing`], [`gesture`]) and send a [`crate::avatar::blow::Blow`] when it lands
//! ([`pounce`]).

mod anatomy;
mod carve;
mod fracture;
mod gesture;
mod harm;
mod kinds;
mod limbs;
mod pounce;
mod raise;
mod slay;
mod slump;
mod timing;
pub mod tune;

#[cfg(test)]
mod tests_attack;
#[cfg(test)]
mod tests_harm;
#[cfg(test)]
mod tests_limbs;

use bevy::prelude::*;

pub use harm::{Dead, harm};
pub use limbs::Broken;
pub use slay::{Carved, slay};

use crate::screens::{in_control, playing};

/// Registers the harm of blows: before the scan, so a kill leaves the scan's wants at once. The attacks
/// run after the senses set the stances and before the walk, the telegraphs after the walk posed the
/// parts, the collapse and the carving beside them.
pub(super) fn plugin(app: &mut App) {
    let scan = super::scan::scan;
    let walk = super::walk::walk;
    app.add_systems(
        Update,
        (
            harm::harm
                .run_if(on_message::<crate::avatar::strike::Hit>)
                .before(scan),
            raise::raise.after(scan),
            pounce::attack
                .after(super::senses::sense)
                .before(walk)
                .run_if(any_with_component::<super::beast::Beast>),
            gesture::gesture.after(walk).run_if(
                any_with_component::<timing::Attacking>
                    .or_else(any_with_component::<gesture::Flinch>),
            ),
            slump::slump
                .after(walk)
                .run_if(any_with_component::<slump::Slump>),
            carve::carve.run_if(in_control),
        )
            .run_if(playing),
    );
}
