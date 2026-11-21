//! The one spatial path every sound of the world takes to the ears.
//!
//! A source has a place in the world and a reach. On the game thread `spot` works out, once when
//! the sound starts, how it arrives at the [`Listener`]: its pressure after `1 / r` and the
//! density of the air, its bearing, its delay over the speed of sound of the planet, the corner
//! of the low-pass the air makes after ISO 9613-1 (`iso`, `air`), what a hill or the water
//! surface takes, and what the body carries in a vacuum. On the audio thread an [`Ear`] turns it
//! into two channels. A moving source of its own, the ship, aims an ear every few milliseconds
//! through its delay line instead.
//!
//! The interface and the music stay flat: they have no place in the world.

mod air;
pub(super) mod ear;
pub(super) mod iso;
pub(super) mod listen;
pub(super) mod room;
pub(super) mod spot;
pub mod tune;

#[cfg(test)]
mod tests_air;
#[cfg(test)]
mod tests_cabin;
#[cfg(test)]
mod tests_ear;
#[cfg(test)]
mod tests_room;
#[cfg(test)]
mod tests_spot;

pub use air::{Absorb, Air};
pub use ear::{Ear, around};
pub use listen::{Ground, Listener, hidden};
pub use room::{Room, Seat};
pub use spot::{Between, Origin, SPOT, Sound, Spot, spot};

use bevy::prelude::*;

/// Registers the sounds of the world and the listener.
pub(super) fn plugin(app: &mut App) {
    app.add_message::<Sound>().init_resource::<Listener>();
}
