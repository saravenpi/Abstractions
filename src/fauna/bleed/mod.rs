//! Bleeding: a blow that opens an animal makes a [`Wound`], and the blood leaves it as real drops.
//!
//! The layers of the struck part decide whether the blow got through to the flesh ([`open`]): a blunt
//! one bruises and makes no wound. A wound throws a burst at once ([`cut`]) and then bleeds each frame
//! at a flow that follows the pressure of the animal, the size of the cut and how far it clotted
//! ([`wound`], [`drain`]); the blood lost lowers the health, and a stuck spear plugs the wound. The
//! drops fly in [`crate::effects`] and a drop that lands lays a stain ([`stain`]) in the colour the
//! life of the planet gives the blood of its species ([`crate::life::blood`]).

mod chips;
mod cut;
mod drain;
mod open;
mod stain;
mod throw;
pub mod tune;
mod wound;

#[cfg(test)]
mod tests_open;
#[cfg(test)]
mod tests_wound;

use bevy::prelude::*;

pub use wound::{Circuit, Cut, Wound};

use crate::avatar::strike::Hit;
use crate::effects::Landed;
use crate::screens::playing;

/// Registers the systems of bleeding: the cut before the harm of the blow, the drain after it.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            cut::cut
                .run_if(on_message::<Hit>)
                .before(super::attack::harm),
            drain::drain
                .after(super::attack::harm)
                .run_if(any_with_component::<Wound>),
            stain::lay.run_if(on_message::<Landed>),
        )
            .run_if(playing),
    );
}
