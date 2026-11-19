//! The food web of a planet, after the niche model, and the assembly of its whole life.

use super::World;
use super::{Life, odds};
use crate::universe::{PlanetSpec, StarSpec};

/// Returns the whole life of `planet` around `star`, or nothing on a dead world.
///
/// Phase 0 stub: no planet lives yet.
pub fn assemble(planet: &PlanetSpec, star: &StarSpec) -> Option<Life> {
    let world = World::of(planet, star);
    odds::rung(&world, planet.seed)?;
    None
}
