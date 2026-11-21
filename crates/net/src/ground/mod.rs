//! The ground of a world: the things players put down, kept until someone takes them. A server
//! keeps it for everyone, and a game alone keeps its own the same way.
//!
//! The rule that bounds it, the same everywhere:
//!
//! - A heap put down within [`MERGE`] metres of a heap of the same material on the same body joins
//!   it: one heap, the masses added.
//! - The surface of a body is cut into cubes of [`CELL`] metres. A cube holds at most [`CELL_MAX`]
//!   things, and a world at most [`GROUND_MAX`]. In a full cube a heap joins the heap of its
//!   material that lies in the cube, wherever in it. Anything else is refused and stays with the
//!   player who tried to put it down.
//! - Nothing on the ground wears, rots or leaves by itself: the world is persistent.

mod rule;
#[cfg(test)]
pub(crate) mod tests;

use serde::{Deserialize, Serialize};

use crate::wire::Lying;

/// Side of the cube of the ground that holds at most [`CELL_MAX`] things, in metres.
pub const CELL: f64 = 32.0;

/// Most things one cube of [`CELL`] metres holds.
pub const CELL_MAX: usize = 24;

/// Most things the ground of a world holds.
pub const GROUND_MAX: usize = 16_384;

/// Distance within which a heap joins a heap of the same material, in metres.
pub const MERGE: f64 = 2.0;

/// Mass under which nothing of a heap is left, in kilograms.
pub const CRUMB: f32 = 1.0e-4;

/// The things on the ground and the id the next one gets.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Pile {
    /// The things, oldest first.
    pub lying: Vec<Lying>,
    /// The id the next thing gets. Ids start at 1.
    pub next: u64,
}

/// What became of a thing put down.
#[derive(Debug, Clone, PartialEq)]
pub enum Placed {
    /// It lies on its own, with its new id.
    Laid(Lying),
    /// It joined a heap of its material, which is now this.
    Joined(Lying),
    /// The ground there is full: it stays with its player.
    Refused,
}

/// What a take got off the ground.
#[derive(Debug, Clone, PartialEq)]
pub struct Took {
    /// The thing, or the part of the heap that was taken.
    pub got: Lying,
    /// What is left of the heap on the ground, or nothing when it is gone.
    pub left: Option<Lying>,
}

impl Pile {
    /// Returns the pile of `lying`, at most [`GROUND_MAX`] of them, the next id past the highest.
    pub fn new(mut lying: Vec<Lying>) -> Self {
        lying.truncate(GROUND_MAX);
        let next = lying.iter().map(|l| l.id).max().unwrap_or(0) + 1;
        Self { lying, next }
    }

    /// Returns the thing with `id`.
    pub fn get(&self, id: u64) -> Option<&Lying> {
        self.lying.iter().find(|l| l.id == id)
    }

    /// Puts `lying` in place of the thing with its id, or adds it, as a game does with what a
    /// server tells.
    pub fn told(&mut self, lying: Lying) {
        match self.lying.iter_mut().find(|l| l.id == lying.id) {
            Some(old) => *old = lying,
            None => self.lying.push(lying),
        }
    }

    /// Forgets the thing with `id`, as a game does when a server says it left the ground.
    pub fn forget(&mut self, id: u64) {
        self.lying.retain(|l| l.id != id);
    }
}
