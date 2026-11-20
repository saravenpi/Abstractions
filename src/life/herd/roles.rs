//! Roles in a herd: one adult keeps watch for the others.
//!
//! In herds of grazing mammals a few members scan while the others feed, and their alarm sets off
//! the whole group (Elgar 1989, "Predator vigilance and group size in mammals and birds", Biological
//! Reviews 64, 13-33). A herd here has one lookout, an adult picked by a roll seeded with the herd's
//! identity, so every player sees the same one.

use super::Herd;
use crate::universe::hash::{Rng, child};

/// Salt of the lookout roll.
const LOOKOUT: u64 = 0x100C_0075;

/// Returns the slot of the lookout of `herd`: an adult, or slot 0 when the herd holds only young.
pub fn lookout(herd: &Herd) -> u8 {
    let adults = herd.base.saturating_sub(herd.young);
    if adults == 0 {
        return 0;
    }
    let pick = Rng::new(child(herd.id, LOOKOUT)).below(u32::from(adults));
    herd.young + pick as u8
}
