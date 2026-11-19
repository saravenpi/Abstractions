//! Fauna: the creatures of the planet underfoot, spawned from its life, moved by their needs and
//! animated by procedural gaits on rigid parts.

pub mod motion;

use bevy::prelude::*;

use crate::life::SpeciesId;

/// One drawn individual. Its parts are its children, one entity per body part.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Creature {
    /// Its species.
    pub species: SpeciesId,
    /// The herd it belongs to.
    pub herd: u64,
    /// Its place in the herd.
    pub slot: u8,
}

/// Registers the fauna systems.
pub(super) fn plugin(app: &mut App) {
    let _ = app;
}
