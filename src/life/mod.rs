//! Life: whether a planet lives, how far life got there, and every species it holds.
//!
//! Everything here is a pure function of the planet, its star and their seeds, built from plain data
//! and `bevy::math` only, so the same universe seed always gives the same life on every machine.
//! Systems that draw and move creatures live in `crate::fauna`.

pub mod body;
pub mod census;
pub mod chem;
pub mod flora;
pub mod guild;
pub mod herd;
pub mod history;
pub mod hunt;
mod kinds;
pub mod names;
pub mod odds;
pub mod relate;
pub mod signs;
pub mod tissue;
mod types;
pub mod web;
mod world;

pub use kinds::{
    Activity, Build, Chemistry, Energy, Glow, GlowUse, Guild, Habitat, Plan, Relation, Rung,
    Solvent, Structure,
};
pub use types::{
    History, Life, Pattern, Reaction, Role, Species, SpeciesId, Tissue, TissueKind, Tolerance,
};
pub use world::World;

use crate::universe::{PlanetSpec, StarSpec};

/// Returns the life of `planet` around `star`, or nothing on a dead world.
///
/// It must stay under 5 ms: it runs once on the main thread when a planet is entered.
pub fn of(planet: &PlanetSpec, star: &StarSpec) -> Option<Life> {
    web::assemble(planet, star)
}
