//! Slot manipulation operations on placed chests: swapping, merging, splitting, quick-stacking,
//! and sorting using opaque `(kind, data)` equality.

use super::rule::valid_item;
use super::{ChestItem, Chests, ITEM_STACK, PlaceableKind, STACK_MAX};

/// Returns true when `a` and `b` represent identical stackable items.
fn same_stack(a: &ChestItem, b: &ChestItem) -> bool {
    a.kind == ITEM_STACK && a.kind == b.kind && a.data == b.data
}

/// Moves as many units as fit from `src` into `dst`, returning true when both stack together.
fn merge_into(dst: &mut ChestItem, src: &mut ChestItem) -> bool {
    if !same_stack(dst, src) || dst.count >= STACK_MAX {
        return false;
    }
    let add = src.count.min(STACK_MAX - dst.count);
    dst.count += add;
    src.count -= add;
    true
}

/// Returns mutable slots for chest `id`, or nothing when missing or not a chest.
fn chest_slots_mut(chests: &mut Chests, id: u64) -> Option<&mut Vec<Option<ChestItem>>> {
    chests
        .find_mut(id)
        .filter(|p| p.kind == PlaceableKind::Chest)
        .map(|p| &mut p.slots)
}

/// Merges `item` into `merged` stacks or appends remaining full/partial stacks.
fn merge_item(merged: &mut Vec<ChestItem>, mut item: ChestItem) {
    if item.kind != ITEM_STACK {
        merged.push(item);
        return;
    }
    for dst in merged.iter_mut() {
        merge_into(dst, &mut item);
    }
    while item.count > 0 {
        let put = item.count.min(STACK_MAX);
        merged.push(ChestItem {
            kind: ITEM_STACK,
            count: put,
            data: item.data.clone(),
        });
        item.count -= put;
    }
}

impl Chests {
    /// Swaps slots `from` and `to` of chest `id`, or merges matching stacks up to [`STACK_MAX`].
    pub fn swap_or_merge(&mut self, id: u64, from: u8, to: u8) -> bool {
        let Some(slots) = chest_slots_mut(self, id) else {
            return false;
        };
        let (from, to) = (usize::from(from), usize::from(to));
        if from >= slots.len() || to >= slots.len() || from == to {
            return false;
        }
        let Some(mut src) = slots[from].take() else {
            if slots[to].is_none() {
                return false;
            }
            slots.swap(from, to);
            return true;
        };
        if let Some(dst) = slots[to].as_mut()
            && merge_into(dst, &mut src)
        {
            slots[from] = (src.count > 0).then_some(src);
            return true;
        }
        slots[from] = Some(src);
        slots.swap(from, to);
        true
    }

    /// Takes half rounded up from a stack in `slot` of chest `id`, or the whole non-stack item.
    pub fn split_slot(&mut self, id: u64, slot: u8) -> Option<ChestItem> {
        let slots = chest_slots_mut(self, id)?;
        let cell = slots.get_mut(usize::from(slot))?;
        let item = cell.as_mut()?;
        if item.kind == ITEM_STACK && item.count > 1 {
            let half = item.count.div_ceil(2);
            item.count -= half;
            return Some(ChestItem {
                kind: ITEM_STACK,
                count: half,
                data: item.data.clone(),
            });
        }
        cell.take()
    }

    /// Inserts `item` into chest `id`, filling partial matching stacks first then an empty slot,
    /// and returns any unplaced remainder.
    pub fn quick_stack(&mut self, id: u64, mut item: ChestItem) -> Option<ChestItem> {
        let Some(slots) = valid_item(&item)
            .then_some(())
            .and_then(|()| chest_slots_mut(self, id))
        else {
            return Some(item);
        };
        if item.kind == ITEM_STACK {
            for dst in slots.iter_mut().flatten() {
                merge_into(dst, &mut item);
            }
        }
        if item.count > 0
            && let Some(empty) = slots.iter_mut().find(|s| s.is_none())
        {
            *empty = Some(item);
            return None;
        }
        (item.count > 0).then_some(item)
    }

    /// Compacts matching stacks and sorts all items in chest `id` by `(kind, data, count desc)`.
    pub fn sort_slots(&mut self, id: u64) -> bool {
        let Some(slots) = chest_slots_mut(self, id) else {
            return false;
        };
        let mut merged = Vec::with_capacity(slots.len());
        for item in slots.iter_mut().filter_map(Option::take) {
            merge_item(&mut merged, item);
        }
        merged.sort_by(|a, b| {
            a.kind
                .cmp(&b.kind)
                .then_with(|| a.data.cmp(&b.data))
                .then_with(|| b.count.cmp(&a.count))
        });
        for (dst, item) in slots.iter_mut().zip(merged) {
            *dst = Some(item);
        }
        true
    }
}
