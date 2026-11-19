//! Whether a planet lives, and how far evolution got there.

use super::{Rung, World};

/// Returns how far life got on a planet of `world` whose life rolls from `seed`, or nothing on a dead world.
///
/// Phase 0 stub: every planet is dead until the generator lands.
pub fn rung(world: &World, seed: u64) -> Option<Rung> {
    let _ = (world, seed);
    None
}
