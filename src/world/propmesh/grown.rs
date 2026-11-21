//! Grown plants: parametric low-poly meshes of the eight growth architectures, turned by the dials of a
//! producer group of a living planet.
//!
//! A grown plant rides on a carrier [`Plant`] whose collider and room suit it, and names its look by a
//! variant past [`VARIANTS`]: the page of the [`book`] that holds its mesh.

mod bits;
mod book;
mod bulb;
mod kit;
mod low;
mod rosette;
mod tall;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_joined;
mod tree;

pub(in crate::world::propmesh) use book::soup;
pub use book::{edition, look, page, tris};

use super::{Plant, VARIANTS};
use crate::life::flora::Dials;
use crate::world::shapes::FlatMesh;
use kit::Kit;

/// Ages a species is built at: young, grown and old.
pub const AGES: usize = 3;

/// The look of one grown plant: the dials of its species and the age it is built at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// Dials of its species at this stage of the year.
    pub dials: Dials,
    /// Age, below [`AGES`]: 0 young, 1 grown, 2 old.
    pub variant: u8,
}

/// How a growth architecture stands in the scatter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Habit {
    /// The plant whose collider, room and pass it takes.
    pub carrier: Plant,
    /// Height in metres its mesh is built at; its placement scale is its height over this.
    pub reference: f32,
}

/// The habit of every architecture, in architecture order.
const HABITS: [Habit; 8] = [
    Habit {
        carrier: Plant::Oak,
        reference: 4.0,
    },
    Habit {
        carrier: Plant::Bush,
        reference: 1.0,
    },
    Habit {
        carrier: Plant::MushroomRing,
        reference: 1.0,
    },
    Habit {
        carrier: Plant::MushroomRing,
        reference: 1.5,
    },
    Habit {
        carrier: Plant::MushroomRing,
        reference: 0.6,
    },
    Habit {
        carrier: Plant::Oak,
        reference: 4.0,
    },
    Habit {
        carrier: Plant::Sponge,
        reference: 4.0,
    },
    Habit {
        carrier: Plant::Oak,
        reference: 2.5,
    },
];

/// Returns the habit of architecture `arch`.
pub fn habit(arch: u8) -> Habit {
    HABITS[usize::from(arch.min(7))]
}

/// Returns true when `variant` names a page of the book rather than a shared mesh of home.
pub fn grown(variant: u16) -> bool {
    usize::from(variant) >= VARIANTS
}

/// Builds the mesh of `look` at the reference height of its architecture, base at the origin.
pub fn build(look: &Look) -> FlatMesh {
    let mut kit = Kit::new(look);
    match look.dials.architecture {
        0 => tree::tree(&mut kit),
        1 => rosette::rosette(&mut kit),
        2 => low::mat(&mut kit),
        3 => low::vine(&mut kit),
        4 => bulb::bulb(&mut kit),
        5 => tall::stalk(&mut kit),
        6 => bulb::kelp(&mut kit),
        _ => tall::lattice(&mut kit),
    }
    kit.mesh
}
