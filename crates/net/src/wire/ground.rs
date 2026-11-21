//! Things on the ground on the wire: a tool or a heap of material a player put down, as the server
//! keeps it and tells it.
//!
//! The server reads the head of a record (its id, where it lies, its kind, its mass and what it is
//! made of, to merge heaps) and never the bytes the game rebuilds the thing from.

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;
use crate::log::PlayerId;

/// Kind of a thing that is a made tool.
pub const TOOL: u8 = 0;

/// Kind of a thing that is a heap of one material.
pub const HEAP: u8 = 1;

/// Most bytes of `material` and `item` together the server takes. A tool is about 160.
pub const LYING_BYTES: usize = 512;

/// Most things in one message of the ground a player is told on joining.
pub const GROUND_CHUNK: usize = 512;

/// Heaviest thing the server takes, in kilograms.
pub const LYING_MASS: f32 = 1.0e4;

/// Writes bytes as hexadecimal text in human-readable formats and as plain bytes in binary ones,
/// so a world file of the game stays readable.
mod hex {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    /// Writes `bytes` as hexadecimal text, or as a sequence of bytes.
    pub fn serialize<S: Serializer>(bytes: &[u8], to: S) -> Result<S::Ok, S::Error> {
        if !to.is_human_readable() {
            return bytes.serialize(to);
        }
        let text: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        to.serialize_str(&text)
    }

    /// Reads bytes written by [`serialize`].
    pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<Vec<u8>, D::Error> {
        if !from.is_human_readable() {
            return Vec::<u8>::deserialize(from);
        }
        let text = String::deserialize(from)?;
        let pairs = text.as_bytes().chunks(2);
        let byte = |pair: &[u8]| {
            let pair = std::str::from_utf8(pair).ok().filter(|p| p.len() == 2)?;
            u8::from_str_radix(pair, 16).ok()
        };
        let bytes: Option<Vec<u8>> = pairs.map(byte).collect();
        bytes.ok_or_else(|| D::Error::custom("not hexadecimal"))
    }
}

/// One thing that lies on the ground of a body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Lying {
    /// Its number in the world. The server gives it; a drop sends 0.
    pub id: u64,
    /// The player whose drop made or last grew it. A server overwrites it with the sender's id.
    pub by: PlayerId,
    /// The number the dropping game gave its drop, so it knows the answer; 0 when no drop of a
    /// game waits for it.
    pub tag: u32,
    /// The body it lies on.
    pub body: BodyId,
    /// Where it lies, in metres from the centre of the body in its turning frame.
    pub at: [f64; 3],
    /// What it is: [`TOOL`] or [`HEAP`].
    pub kind: u8,
    /// What it is made of, as bytes of the game. Two heaps with the same bytes are one material.
    #[serde(with = "hex")]
    pub material: Vec<u8>,
    /// Its mass in kilograms.
    pub mass: f32,
    /// How worn it is, 0 new to 1 broken; 0 for a heap.
    pub wear: f32,
    /// The rest of what the game rebuilds it from, as bytes of the game.
    #[serde(with = "hex")]
    pub item: Vec<u8>,
    /// Universe time it was put down, in seconds.
    pub since: f64,
}

impl Lying {
    /// Returns true when a server may keep it: every number finite, a mass above zero and within
    /// [`LYING_MASS`], a known kind and at most [`LYING_BYTES`] of bytes.
    pub fn sane(&self) -> bool {
        let numbers = self.at.iter().all(|x| x.is_finite()) && self.wear.is_finite();
        let mass = self.mass.is_finite() && self.mass > 0.0 && self.mass <= LYING_MASS;
        let bytes = self.material.len() + self.item.len() <= LYING_BYTES;
        numbers && mass && bytes && matches!(self.kind, TOOL | HEAP)
    }
}
