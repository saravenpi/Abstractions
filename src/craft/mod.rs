//! Crafting: what harvested tissues and stones are made of, and the tools built from them.
//!
//! Everything here is pure model code: plain data and functions of the planet and the materials a
//! player carries, so a server can run the same numbers and agree with every client.

pub mod build;
pub mod matter;

use bevy::prelude::*;

/// Registers the crafting area. The model has no systems yet: the workshop and the swings that
/// read it come in later waves.
pub(super) fn plugin(_app: &mut App) {}
