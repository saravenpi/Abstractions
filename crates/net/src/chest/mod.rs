//! Placed props of a world: chests, torches, and the slot rules that bound them.

pub mod rule;
mod slots;
#[cfg(test)]
mod slots_tests;
#[cfg(test)]
mod tests;

pub use rule::{valid_dir, valid_item, valid_prop};

use serde::{Deserialize, Serialize};

use crate::ids::BodyId;

/// Number of storage slots in a placed chest (6x4 grid).
pub const CHEST_SLOTS: usize = 24;

/// Maximum discrete units held in a single stacked item slot.
pub const STACK_MAX: u16 = 64;

/// Maximum placed chests and torches retained in a single world.
pub const CHEST_MAX_PROPS: usize = 512;

/// Maximum serialized payload bytes for one item inside a chest slot.
pub const CHEST_ITEM_BYTES: usize = 1024;

/// Item kind tag for a stackable material, bullet, charge, wood, or placeable.
pub const ITEM_STACK: u8 = 0;

/// Item kind tag for a crafted tool.
pub const ITEM_TOOL: u8 = 1;

/// Item kind tag for a crafted firearm.
pub const ITEM_GUN: u8 = 2;

/// Item kind tag for a piece of armor.
pub const ITEM_ARMOR: u8 = 3;

/// Kind of placeable prop placed on a body surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaceableKind {
    /// A 24-slot storage container.
    Chest,
    /// A surface light source with no storage slots.
    Torch,
}

/// One opaque item entry inside a chest slot, kept game-type-agnostic on the wire.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChestItem {
    /// Item kind tag: [`ITEM_STACK`], [`ITEM_TOOL`], [`ITEM_GUN`], or [`ITEM_ARMOR`].
    pub kind: u8,
    /// Unit count (`1..=STACK_MAX` for stacks, `1` for tools and guns).
    pub count: u16,
    /// Game-encoded item payload up to [`CHEST_ITEM_BYTES`] bytes.
    pub data: Vec<u8>,
}

/// Sparse TOML- and postcard-safe serializer for chest slot vectors without null elements.
mod serde_slots {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{CHEST_SLOTS, ChestItem};

    /// Compact representation storing total slot count and occupied `(slot, item)` pairs.
    #[derive(Serialize, Deserialize)]
    struct SparseSlots {
        /// Total slot vector length (`0` for torch, [`CHEST_SLOTS`] for chest).
        len: u8,
        /// Occupied slot indices and their stored items.
        items: Vec<(u8, ChestItem)>,
    }

    /// Serializes `slots` sparsely so both TOML and postcard round-trip losslessly.
    pub fn serialize<S: Serializer>(slots: &[Option<ChestItem>], to: S) -> Result<S::Ok, S::Error> {
        let len = u8::try_from(slots.len()).unwrap_or(CHEST_SLOTS as u8);
        let items = slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| Some((u8::try_from(i).ok()?, slot.clone()?)))
            .collect();
        SparseSlots { len, items }.serialize(to)
    }

    /// Deserializes a sparse slot table back into `Vec<Option<ChestItem>>`.
    pub fn deserialize<'de, D: Deserializer<'de>>(
        from: D,
    ) -> Result<Vec<Option<ChestItem>>, D::Error> {
        let sparse = SparseSlots::deserialize(from)?;
        let len = usize::from(sparse.len).min(CHEST_SLOTS);
        let mut out = vec![None; len];
        for (idx, item) in sparse.items {
            if let Some(cell) = out.get_mut(usize::from(idx)) {
                *cell = Some(item);
            }
        }
        Ok(out)
    }
}

/// A placed chest or torch anchored on a body in the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedProp {
    /// World-unique prop identifier assigned by the server starting at 1.
    pub id: u64,
    /// Body the prop rests on.
    pub body: BodyId,
    /// Unit surface direction from the body centre in its rotating frame.
    pub dir: [f32; 3],
    /// Yaw orientation around the local surface normal, in radians.
    pub yaw: f32,
    /// Whether the prop is a chest or a torch.
    pub kind: PlaceableKind,
    /// Storage slots ([`CHEST_SLOTS`] entries for a chest, empty for a torch).
    #[serde(with = "serde_slots")]
    pub slots: Vec<Option<ChestItem>>,
}

/// Collection of placed props in a world and the next id to assign.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Chests {
    /// All placed props in placement order.
    pub props: Vec<PlacedProp>,
    /// Next prop id to assign; ids start at 1.
    pub next_id: u64,
}

impl Chests {
    /// Returns a collection built from `props`, bounded to [`CHEST_MAX_PROPS`], with `next_id`
    /// past the highest existing id.
    pub fn new(mut props: Vec<PlacedProp>) -> Self {
        props.truncate(CHEST_MAX_PROPS);
        let next_id = props.iter().map(|p| p.id).max().unwrap_or(0) + 1;
        Self { props, next_id }
    }

    /// Returns a shared reference to the placed prop with `id`, if present.
    pub fn find(&self, id: u64) -> Option<&PlacedProp> {
        self.props.iter().find(|p| p.id == id)
    }

    /// Returns a mutable reference to the placed prop with `id`, if present.
    pub fn find_mut(&mut self, id: u64) -> Option<&mut PlacedProp> {
        self.props.iter_mut().find(|p| p.id == id)
    }

    /// Replaces the prop with `prop.id` or appends it, as a client applies server snapshots.
    pub fn told(&mut self, prop: PlacedProp) {
        self.next_id = self.next_id.max(prop.id.saturating_add(1));
        match self.find_mut(prop.id) {
            Some(old) => *old = prop,
            None => self.props.push(prop),
        }
    }

    /// Removes the prop with `id`, as a client applies server break events.
    pub fn forget(&mut self, id: u64) {
        self.props.retain(|p| p.id != id);
    }
}
