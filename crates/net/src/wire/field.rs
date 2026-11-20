//! Records of the field besides hunting: blows on robots, wounds, impacts, attacks of animals
//! and taken rocks. The server keeps wounds and rocks, relays the rest, and stamps the player.

use serde::{Deserialize, Serialize};

use super::hunt::AnimalKey;
use crate::ids::BodyId;
use crate::log::PlayerId;

/// Result class of a swing: it cut or crushed through the outer layer.
pub const CLEAN: u8 = 0;

/// Result class of a swing: it glanced off.
pub const DEFLECTED: u8 = 1;

/// Result class of a swing: it was stopped by a layer.
pub const STOPPED: u8 = 2;

/// Result class of a swing: it hit nothing.
pub const MISS: u8 = 3;

/// A blow on the robot of a player. Only the victim hears it, and its client decides what the blow
/// does to its robot and tells the result in its records.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Blow {
    /// The player whose robot was hit.
    pub victim: PlayerId,
    /// Contact point in metres from the centre of the victim's body, in axes whose Y is the local up.
    pub at: [f32; 3],
    /// Energy delivered, in joules.
    pub joules: f32,
    /// True when an edge or a point carried it.
    pub cut: bool,
    /// Unit direction the blow travelled, in the same axes.
    pub dir: [f32; 3],
    /// Momentum delivered, in kilogram metres per second.
    pub momentum: f32,
}

/// A wound on a part of an animal: every client lets it bleed the same way from the same time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Wound {
    /// Index of the part.
    pub part: u8,
    /// Depth in metres.
    pub depth: f32,
    /// Width in metres.
    pub width: f32,
    /// Which major vessel it opened, 0 for none.
    pub vessel: u8,
    /// Universe time it opened, in seconds.
    pub opened: f64,
    /// True once a spear plugs it.
    pub plugged: bool,
}

/// What a swing or a throw did where it landed, for the sound and the chips.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Impact {
    /// The body it happened on.
    pub body: BodyId,
    /// Where, in metres in the turning frame of the body.
    pub at: [f64; 3],
    /// Energy delivered, in joules.
    pub joules: f32,
    /// True when an edge or a point carried it.
    pub cut: bool,
    /// Class of the outer layer struck, as the game numbers them.
    pub outer: u8,
    /// Class of the tool, as the game numbers them.
    pub tool: u8,
    /// How it went: [`CLEAN`], [`DEFLECTED`], [`STOPPED`] or [`MISS`].
    pub result: u8,
}

/// An attack an animal starts, so that its wind-up, lunge and recovery play in step everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Attack {
    /// The animal.
    pub animal: AnimalKey,
    /// Which attack, as the game numbers them.
    pub kind: u8,
    /// The player it goes for, or nothing for another animal.
    pub target: Option<PlayerId>,
    /// Universe time it starts, in seconds.
    pub started: f64,
}

/// A rock taken from the ground. It stays gone for everyone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rock {
    /// The planet it lay on.
    pub planet: BodyId,
    /// Its id on that planet.
    pub id: u64,
}

/// Returns true when every number is finite.
pub fn finite(numbers: &[f32]) -> bool {
    numbers.iter().all(|x| x.is_finite())
}

/// Wound with its animal, as the room keeps it.
pub type Opened = (AnimalKey, Wound);
