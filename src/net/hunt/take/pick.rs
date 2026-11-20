//! Finding the animal an event is about among the animals drawn, and the bit of a part.

use abstractions_net::wire::AnimalKey;
use bevy::prelude::*;

use crate::fauna::{Beast, Lead};

/// Returns the animal `animal` that is drawn, or the one that lies as the carcass of `key`.
pub(super) fn find<'a>(
    beasts: &'a mut Query<(Entity, &mut Beast)>,
    animal: &AnimalKey,
    key: Option<u64>,
) -> Option<(Entity, Mut<'a, Beast>)> {
    beasts.iter_mut().find(|(_, beast)| match beast.lead {
        Lead::Herd { id, slot } => id == animal.herd && slot == animal.slot,
        Lead::Carcass { key: lying, .. } => Some(lying) == key,
        Lead::Visit { .. } => false,
    })
}

/// Returns the bit of part `part` in a mask of parts, or nothing for a part past the 64th.
pub(super) fn bit(part: u8) -> u64 {
    1u64.checked_shl(u32::from(part)).unwrap_or(0)
}
