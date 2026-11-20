//! Hunting on the wire: hits on animals, kills, carcasses, stuck spears and held items.
//!
//! The server relays and stores these and never simulates them. It stamps the player itself, so
//! a player cannot speak for another. An item travels as the bytes of its craft recipe, which
//! the server never reads.

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;
use crate::log::PlayerId;

/// Most bytes of an item the server takes. A recipe is about 70 bytes.
pub const ITEM_MAX: usize = 96;

/// Parts an animal has at most: a part is a bit of a `u64`.
pub const PARTS_MAX: u8 = 64;

/// Most carcasses the server keeps. The oldest goes first.
pub const CARCASSES_MAX: usize = 1024;

/// Most stuck spears the server keeps. The oldest goes first.
pub const SPEARS_MAX: usize = 256;

/// Seconds before a kill during which a hit makes its player share the loot.
pub const CREDIT_WINDOW: f64 = 30.0;

/// Shortest and longest time in seconds a carcass is kept.
pub const LASTS: (f32, f32) = (60.0, 86_400.0);

/// Seconds a kill may differ from the clock of the server and still count at its own time.
pub const SKEW: f64 = 5.0;

/// One animal: member `slot` of herd `herd` on `body`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnimalKey {
    /// The body the herd lives on.
    pub body: BodyId,
    /// Identity of the herd.
    pub herd: u64,
    /// Slot of the member in the herd.
    pub slot: u8,
}

/// A blow that landed on a part of an animal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    /// The animal struck.
    pub animal: AnimalKey,
    /// Index of the part struck.
    pub part: u8,
    /// Energy that arrived, in joules.
    pub joules: f32,
    /// True for a crushing blow.
    pub blunt: bool,
}

/// What a player says of the kill of an animal.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Kill {
    /// The animal killed.
    pub animal: AnimalKey,
    /// Index of its species on its planet.
    pub species: u16,
    /// Unit direction from the centre of the body to where it lies.
    pub at: [f64; 3],
    /// Universe time of the death in seconds.
    pub since: f64,
    /// How cleanly it was killed: 0 clean, 1 good, 2 poor.
    pub grade: u8,
    /// Seconds the carcass and the gap in its herd last.
    pub lasts: f32,
}

/// What is left of a killed animal, the same for every player.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Carcass {
    /// The animal it was.
    pub animal: AnimalKey,
    /// Index of its species on its planet.
    pub species: u16,
    /// Unit direction from the centre of the body to where it lies.
    pub at: [f64; 3],
    /// Universe time of the death in seconds.
    pub since: f64,
    /// How cleanly it was killed: 0 clean, 1 good, 2 poor.
    pub grade: u8,
    /// Seconds it lasts from `since`.
    pub lasts: f32,
    /// The parts taken, a bit for each part index.
    pub taken: u64,
    /// The parts broken in the fight, a bit for each part index.
    pub broken: u64,
    /// The players who hurt it in the [`CREDIT_WINDOW`] before it died.
    pub credited: Vec<PlayerId>,
}

/// A spear in the thrower's name: its owner and a number the owner counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpearKey {
    /// The player who threw it. A server overwrites it with the sender's id.
    pub owner: PlayerId,
    /// Number the owner gave it.
    pub tag: u32,
}

/// A spear that stays in an animal, and then in its carcass, until somebody pulls it out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spear {
    /// Whose it is.
    pub key: SpearKey,
    /// The animal it stands in.
    pub animal: AnimalKey,
    /// Where its point is, in metres in the frame of the animal.
    pub at: [f32; 3],
    /// Unit vector along the spear, from its butt to its point.
    pub dir: [f32; 3],
    /// The item, as the bytes of its recipe.
    pub item: Vec<u8>,
}
