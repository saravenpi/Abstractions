//! The rocks of the world that the robot can break up: a stable id for every rock prop, the set of
//! ids already taken, and the message that says one was.
//!
//! A rock is a big prop. Its id comes from the chunk it stands in and from where it stands, so it is
//! the same on every machine and every run for the same world. A taken rock is left out when its chunk
//! is built, so it never comes back, and the set is saved in the world file.

mod id;
mod save;
mod stand;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::f32::consts::PI;
use std::sync::Arc;

use bevy::prelude::*;

pub use self::id::{id_of, is_rock, key, spare};
pub use self::save::{load, store};
pub use self::stand::Stand;
pub use super::propmesh::Plant;
use super::props::extent;
use crate::universe::BodyId;

/// The identity of one rock prop of a planet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RockId(pub u64);

impl RockId {
    /// Returns the id as 16 hexadecimal digits, the form the world file keeps: a TOML integer holds
    /// only 63 bits.
    pub fn text(self) -> String {
        format!("{:016x}", self.0)
    }

    /// Returns the id written by [`RockId::text`], or nothing for any other text.
    pub fn parse(text: &str) -> Option<Self> {
        u64::from_str_radix(text, 16).ok().map(Self)
    }
}

/// One rock prop of a loaded chunk, in the frame of its chunk.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rock {
    /// Its identity.
    pub id: RockId,
    /// Which rock it is.
    pub plant: Plant,
    /// Its scale.
    pub scale: f32,
    /// Where it meets the ground.
    pub at: Vec3,
    /// The middle of its collider.
    pub mid: Vec3,
}

impl Rock {
    /// Returns the mass in kilograms of the rock when its stone weighs `density` kilograms per cubic
    /// metre. The rock fills the ellipsoid inscribed in its collider, which is a cylinder.
    pub fn mass(&self, density: f32) -> f32 {
        let [radius, height] = extent(self.plant, self.scale);
        2.0 / 3.0 * PI * radius * radius * height * density
    }
}

/// The rocks the players have taken from every planet of the world.
#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Taken(Arc<HashSet<RockId>>);

impl Taken {
    /// Returns true when rock `id` was taken.
    pub fn has(&self, id: RockId) -> bool {
        self.0.contains(&id)
    }

    /// Notes that rock `id` was taken. Returns false when it already was.
    pub fn add(&mut self, id: RockId) -> bool {
        Arc::make_mut(&mut self.0).insert(id)
    }

    /// Returns the ids as the world file keeps them, in order.
    pub fn to_text(&self) -> Vec<String> {
        let mut ids: Vec<RockId> = self.0.iter().copied().collect();
        ids.sort_unstable();
        ids.into_iter().map(RockId::text).collect()
    }

    /// Returns the set the world file's `text` keeps. Text that is no id is skipped.
    pub fn from_text(text: &[String]) -> Self {
        Self(Arc::new(
            text.iter().filter_map(|t| RockId::parse(t)).collect(),
        ))
    }
}

/// A robot took a rock: it is gone from the world of everyone who sees it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct RockTaken {
    /// The planet the rock stood on.
    pub planet: BodyId,
    /// The rock.
    pub id: RockId,
}
