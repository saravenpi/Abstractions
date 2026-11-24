//! Breakable trees of the world: stable identities, hit tracking, and save persistence.

mod felled;
mod id;
mod save;
mod stand;
#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use self::felled::{Felled, TREE_CHOPS};
pub use self::id::{id_of, is_tree, key, spare_trees, trees_of};
pub use self::save::{load, store};
use super::propmesh::Plant;
use crate::universe::BodyId;

/// Identity of a choppable tree prop in a terrain chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TreeId(pub u64);

impl TreeId {
    /// Returns the id formatted as 16 lowercase hexadecimal digits.
    pub fn text(self) -> String {
        format!("{:016x}", self.0)
    }

    /// Parses an id written by [`TreeId::text`].
    pub fn parse(text: &str) -> Option<Self> {
        u64::from_str_radix(text, 16).ok().map(Self)
    }
}

/// One standing tree prop in a loaded chunk, in chunk-local coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tree {
    /// Stable identity across sessions and machines.
    pub id: TreeId,
    /// Tree species variant.
    pub plant: Plant,
    /// Uniform visual and collider scale factor.
    pub scale: f32,
    /// Ground contact point in chunk space.
    pub at: Vec3,
    /// Trunk tip point in chunk space.
    pub top: Vec3,
    /// Approximate trunk midpoint in chunk space.
    pub mid: Vec3,
    /// Trunk collision radius in metres.
    pub radius: f32,
    /// Cumulative chop hits already taken before removal.
    pub hits: u8,
}

/// Message emitted when a tree takes chop damage or is felled.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeChopped {
    /// Planet where the tree stands.
    pub planet: BodyId,
    /// Identity of the tree.
    pub id: TreeId,
    /// Chop power dealt by this strike.
    pub power: u8,
}
