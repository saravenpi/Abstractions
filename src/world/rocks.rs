//! The rocks of the world that the robot can break up: a stable id for every rock prop, the set of
//! ids already taken, and the message that says one was.
//!
//! A rock is a big prop. Its id comes from the chunk it stands in and from where it stands, so it is
//! the same on every machine and every run for the same world. A rock that was quarried is drawn smaller when
//! its chunk is built, and one that is used up is left out, so it never comes back. The mass taken is
//! saved in the world file.

mod id;
mod save;
mod stand;
mod taken;
#[cfg(test)]
mod tests;
pub mod tune;

use std::f32::consts::PI;

use bevy::prelude::*;

pub use self::id::{id_of, is_rock, key, spare};
pub use self::save::{load, store};
pub use self::stand::Stand;
pub use self::taken::{Quarry, Taken};
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
    /// The share of its mass it had left when it was drawn, from 0 to 1.
    pub left: f32,
}

impl Rock {
    /// Returns the mass in kilograms of the rock at the size it is drawn, when its stone weighs
    /// `density` kilograms per cubic metre. The rock fills the ellipsoid inscribed in its collider, which is a cylinder.
    pub fn mass(&self, density: f32) -> f32 {
        let [radius, height] = extent(self.plant, self.scale);
        2.0 / 3.0 * PI * radius * radius * height * density
    }
}

/// A robot broke a piece off a rock: the rock is smaller, or gone, for everyone who sees it.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct RockTaken {
    /// The planet the rock stood on.
    pub planet: BodyId,
    /// The rock.
    pub id: RockId,
    /// The mass in kilograms this blow took off it.
    pub kilograms: f32,
}
