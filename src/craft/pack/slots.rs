//! Writes the hotbar slots with a sentinel for an empty slot, since TOML has no null.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::SLOTS;

/// The value written for an empty slot.
const EMPTY: u32 = u32::MAX;

/// Writes each slot as its item id, or [`EMPTY`] when the slot holds nothing.
pub fn serialize<S: Serializer>(slots: &[Option<u32>; SLOTS], to: S) -> Result<S::Ok, S::Error> {
    slots.map(|slot| slot.unwrap_or(EMPTY)).serialize(to)
}

/// Reads slots written by [`serialize`].
pub fn deserialize<'de, D: Deserializer<'de>>(from: D) -> Result<[Option<u32>; SLOTS], D::Error> {
    let raw = <[u32; SLOTS]>::deserialize(from)?;
    Ok(raw.map(|id| (id != EMPTY).then_some(id)))
}
