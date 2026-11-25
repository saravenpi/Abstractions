//! Validation and placement operations for placed chests, torches, and chest slots.

use super::{
    CHEST_ITEM_BYTES, CHEST_MAX_PROPS, CHEST_SLOTS, ChestItem, Chests, ITEM_ARMOR, ITEM_GUN,
    ITEM_STACK, ITEM_TOOL, PlaceableKind, PlacedProp, STACK_MAX,
};
use crate::ids::BodyId;

/// Returns true when `dir` is finite, non-zero, and has finite squared length.
pub fn valid_dir(dir: [f32; 3]) -> bool {
    let len_sq: f32 = dir.iter().map(|v| v * v).sum();
    dir.iter().all(|v| v.is_finite()) && len_sq.is_finite() && len_sq > 1.0e-12
}

/// Returns true when `item` has a known kind, valid stack count, and bounded payload bytes.
pub fn valid_item(item: &ChestItem) -> bool {
    if item.data.len() > CHEST_ITEM_BYTES || item.count == 0 {
        return false;
    }
    match item.kind {
        ITEM_STACK => item.count <= STACK_MAX,
        ITEM_TOOL | ITEM_GUN | ITEM_ARMOR => item.count == 1,
        _ => false,
    }
}

/// Returns true when `prop` has a valid direction, finite yaw, and matching slot layout.
pub fn valid_prop(prop: &PlacedProp) -> bool {
    if !valid_dir(prop.dir) || !prop.yaw.is_finite() {
        return false;
    }
    match prop.kind {
        PlaceableKind::Chest => {
            prop.slots.len() == CHEST_SLOTS && prop.slots.iter().flatten().all(valid_item)
        }
        PlaceableKind::Torch => prop.slots.is_empty(),
    }
}

/// Returns `dir` normalized to unit length when not already unit.
fn unit_dir(dir: [f32; 3]) -> [f32; 3] {
    let len = dir.iter().map(|v| v * v).sum::<f32>().sqrt();
    if (len - 1.0).abs() <= 1.0e-5 {
        return dir;
    }
    dir.map(|v| v / len)
}

impl Chests {
    /// Places a new chest or torch on `body` when capacity and geometry are valid.
    pub fn place(
        &mut self,
        body: BodyId,
        dir: [f32; 3],
        yaw: f32,
        kind: PlaceableKind,
    ) -> Option<PlacedProp> {
        if self.props.len() >= CHEST_MAX_PROPS || !valid_dir(dir) || !yaw.is_finite() {
            return None;
        }
        let id = self.next_id.max(1);
        self.next_id = id + 1;
        let slots = match kind {
            PlaceableKind::Chest => vec![None; CHEST_SLOTS],
            PlaceableKind::Torch => Vec::new(),
        };
        let prop = PlacedProp {
            id,
            body,
            dir: unit_dir(dir),
            yaw,
            kind,
            slots,
        };
        self.props.push(prop.clone());
        Some(prop)
    }

    /// Removes and returns the placed prop with `id`, if present.
    pub fn remove(&mut self, id: u64) -> Option<PlacedProp> {
        let idx = self.props.iter().position(|p| p.id == id)?;
        Some(self.props.remove(idx))
    }

    /// Replaces slot `slot` of prop `id` with `item`, returning false if invalid or missing.
    pub fn set_slot(&mut self, id: u64, slot: u8, item: Option<ChestItem>) -> bool {
        if item.as_ref().is_some_and(|i| !valid_item(i)) {
            return false;
        }
        let Some(prop) = self.find_mut(id) else {
            return false;
        };
        let Some(cell) = prop.slots.get_mut(usize::from(slot)) else {
            return false;
        };
        *cell = item;
        true
    }
}
