//! Pure dual-grid chest and pack slot operations: swap, merge, split, Shift-transfer, and
//! quick-stack.

use abstractions_net::chest::{CHEST_SLOTS, ChestItem, ITEM_STACK, STACK_MAX};
use abstractions_net::wire::ChestAct;

use super::{ChestCursor, ChestWorld, OpenChest, codec};
use crate::craft::pack::{BAR_SLOTS, Pack, SlotEntry, TOTAL_SLOTS};

/// Clears `pack.grid[idx]`, pruning any unreferenced tool or firearm record from `pack`.
fn remove_pack_entry(pack: &mut Pack, idx: usize) -> Option<SlotEntry> {
    let entry = pack.grid.get_mut(idx)?.take()?;
    match entry {
        SlotEntry::Tool(id)
            if !pack
                .grid
                .iter()
                .any(|s| matches!(s, Some(SlotEntry::Tool(h)) if *h == id)) =>
        {
            pack.items.retain(|i| i.id != id);
        }
        SlotEntry::Gun(id)
            if !pack
                .grid
                .iter()
                .any(|s| matches!(s, Some(SlotEntry::Gun(h)) if *h == id)) =>
        {
            pack.guns.retain(|g| g.id != id);
        }
        _ => {}
    }
    pack.sync_views();
    Some(entry)
}

/// Deposits `item` into chest `id`, filling partial matching stacks first then an empty slot.
pub fn put_chest_item(world: &mut ChestWorld, id: u64, mut item: ChestItem) -> Option<ChestItem> {
    let slots = world.chests.find(id)?.slots.clone();
    for (i, cell) in slots.iter().enumerate() {
        let Some(dst) = cell.as_ref().filter(|d| {
            item.kind == ITEM_STACK
                && d.kind == ITEM_STACK
                && d.data == item.data
                && d.count < STACK_MAX
        }) else {
            continue;
        };
        let add = item.count.min(STACK_MAX - dst.count);
        let next = ChestItem {
            kind: ITEM_STACK,
            count: dst.count + add,
            data: dst.data.clone(),
        };
        item.count -= add;
        world.emit(ChestAct::SetSlot {
            id,
            slot: i as u8,
            item: Some(next),
        });
        if item.count == 0 {
            return None;
        }
    }
    if let Some(empty) = slots.iter().position(Option::is_none) {
        world.emit(ChestAct::SetSlot {
            id,
            slot: empty as u8,
            item: Some(item),
        });
        return None;
    }
    Some(item)
}

/// Swaps or merges two slots inside chest `id` via [`ChestAct::SetSlot`].
fn swap_chest_slots(world: &mut ChestWorld, id: u64, from: usize, to: usize) -> bool {
    let Some(slots) = world.chests.find(id).map(|p| p.slots.clone()) else {
        return false;
    };
    if from >= slots.len() || to >= slots.len() || from == to {
        return false;
    }
    let (mut src, mut dst) = (slots[from].clone(), slots[to].clone());
    if let (Some(a), Some(b)) = (src.as_mut(), dst.as_mut())
        && a.kind == ITEM_STACK
        && b.kind == ITEM_STACK
        && a.data == b.data
        && b.count < STACK_MAX
    {
        let add = a.count.min(STACK_MAX - b.count);
        b.count += add;
        a.count -= add;
        world.emit(ChestAct::SetSlot {
            id,
            slot: to as u8,
            item: Some(b.clone()),
        });
        world.emit(ChestAct::SetSlot {
            id,
            slot: from as u8,
            item: (a.count > 0).then(|| a.clone()),
        });
        return true;
    }
    world.emit(ChestAct::SetSlot {
        id,
        slot: from as u8,
        item: dst,
    });
    world.emit(ChestAct::SetSlot {
        id,
        slot: to as u8,
        item: src,
    });
    true
}

/// Swaps or merges between chest slot `c_idx` and pack slot `p_idx`.
fn swap_cross(
    world: &mut ChestWorld,
    pack: &mut Pack,
    id: u64,
    (c_idx, p_idx): (usize, usize),
    to_pack: bool,
) -> bool {
    let Some(c_item) = world
        .chests
        .find(id)
        .and_then(|p| p.slots.get(c_idx))
        .cloned()
    else {
        return false;
    };
    if p_idx >= TOTAL_SLOTS {
        return false;
    }
    let p_item = pack.grid[p_idx].and_then(|e| codec::encode_slot(&e, pack));
    if let (Some(mut ci), Some(mut pi)) = (c_item.clone(), p_item.clone())
        && ci.kind == ITEM_STACK
        && pi.kind == ITEM_STACK
        && ci.data == pi.data
    {
        let (src, dst) = if to_pack {
            (&mut ci, &mut pi)
        } else {
            (&mut pi, &mut ci)
        };
        if dst.count < STACK_MAX {
            let add = src.count.min(STACK_MAX - dst.count);
            dst.count += add;
            src.count -= add;
            world.emit(ChestAct::SetSlot {
                id,
                slot: c_idx as u8,
                item: (ci.count > 0).then_some(ci),
            });
            pack.grid[p_idx] = (pi.count > 0)
                .then(|| codec::decode_to_pack(&pi, pack))
                .flatten();
            pack.sync_views();
            return true;
        }
    }
    let _ = remove_pack_entry(pack, p_idx);
    pack.grid[p_idx] = c_item
        .as_ref()
        .and_then(|item| codec::decode_to_pack(item, pack));
    pack.sync_views();
    world.emit(ChestAct::SetSlot {
        id,
        slot: c_idx as u8,
        item: p_item,
    });
    true
}

/// Instantly moves the item at `from` across containers (chest <-> pack).
pub fn shift_transfer(world: &mut ChestWorld, pack: &mut Pack, id: u64, from: ChestCursor) -> bool {
    match from {
        ChestCursor::Chest(idx) => {
            let Some(item) = world
                .chests
                .find(id)
                .and_then(|p| p.slots.get(idx))
                .and_then(Option::as_ref)
                .cloned()
            else {
                return false;
            };
            if item.kind == ITEM_STACK {
                let Some(SlotEntry::Stack {
                    stuff,
                    material,
                    count,
                }) = codec::decode_to_pack(&item, pack)
                else {
                    return false;
                };
                let added = pack.add_stack(stuff, material, count);
                if added == 0 {
                    return false;
                }
                let rem = (count > added).then(|| ChestItem {
                    kind: ITEM_STACK,
                    count: count - added,
                    data: item.data,
                });
                world.emit(ChestAct::SetSlot {
                    id,
                    slot: idx as u8,
                    item: rem,
                });
                return true;
            }
            let Some(free) = (BAR_SLOTS..TOTAL_SLOTS)
                .chain(0..BAR_SLOTS)
                .find(|&i| pack.grid[i].is_none())
            else {
                return false;
            };
            pack.grid[free] = codec::decode_to_pack(&item, pack);
            pack.sync_views();
            world.emit(ChestAct::SetSlot {
                id,
                slot: idx as u8,
                item: None,
            });
            true
        }
        ChestCursor::Pack(idx) => {
            let Some(encoded) = pack
                .grid
                .get(idx)
                .copied()
                .flatten()
                .and_then(|e| codec::encode_slot(&e, pack))
            else {
                return false;
            };
            let before = encoded.count;
            let rem = put_chest_item(world, id, encoded);
            if rem.as_ref().is_some_and(|r| r.count >= before) {
                return false;
            }
            let _ = remove_pack_entry(pack, idx);
            pack.grid[idx] = rem.as_ref().and_then(|r| codec::decode_to_pack(r, pack));
            pack.sync_views();
            true
        }
    }
}

/// Handles left-clicking `target`: Shift transfers immediately; otherwise selects or swaps/merges.
pub fn click_slot(
    world: &mut ChestWorld,
    pack: &mut Pack,
    open: &mut OpenChest,
    target: ChestCursor,
    shift: bool,
) -> bool {
    let Some(id) = open.id else {
        return false;
    };
    if shift {
        open.selected = None;
        return shift_transfer(world, pack, id, target);
    }
    if let Some(from) = open.selected.take() {
        return match (from, target) {
            (a, b) if a == b => false,
            (ChestCursor::Chest(a), ChestCursor::Chest(b)) => swap_chest_slots(world, id, a, b),
            (ChestCursor::Pack(a), ChestCursor::Pack(b)) => pack.swap_slots(a, b),
            (ChestCursor::Chest(c), ChestCursor::Pack(p)) => {
                swap_cross(world, pack, id, (c, p), true)
            }
            (ChestCursor::Pack(p), ChestCursor::Chest(c)) => {
                swap_cross(world, pack, id, (c, p), false)
            }
        };
    }
    let has = match target {
        ChestCursor::Chest(i) => world
            .chests
            .find(id)
            .and_then(|p| p.slots.get(i))
            .is_some_and(Option::is_some),
        ChestCursor::Pack(i) => pack.grid.get(i).is_some_and(Option::is_some),
    };
    open.selected = has.then_some(target);
    false
}

/// Right-click splits a stack at `target` in half into the first empty slot of the same container.
pub fn split_slot(
    world: &mut ChestWorld,
    pack: &mut Pack,
    open: &mut OpenChest,
    target: ChestCursor,
) -> bool {
    let Some(id) = open.id else { return false };
    open.selected = None;
    match target {
        ChestCursor::Chest(idx) => {
            let Some(slots) = world.chests.find(id).map(|p| p.slots.clone()) else {
                return false;
            };
            let Some(item) = slots
                .get(idx)
                .and_then(Option::as_ref)
                .filter(|i| i.kind == ITEM_STACK && i.count > 1)
            else {
                return false;
            };
            let Some(free) = (0..CHEST_SLOTS).find(|&i| i != idx && slots.get(i) == Some(&None))
            else {
                return false;
            };
            let half = item.count.div_ceil(2);
            world.emit(ChestAct::SetSlot {
                id,
                slot: idx as u8,
                item: Some(ChestItem {
                    kind: ITEM_STACK,
                    count: item.count - half,
                    data: item.data.clone(),
                }),
            });
            world.emit(ChestAct::SetSlot {
                id,
                slot: free as u8,
                item: Some(ChestItem {
                    kind: ITEM_STACK,
                    count: half,
                    data: item.data.clone(),
                }),
            });
            true
        }
        ChestCursor::Pack(idx) => {
            let Some(free) = (BAR_SLOTS..TOTAL_SLOTS)
                .chain(0..BAR_SLOTS)
                .find(|&i| i != idx && pack.grid[i].is_none())
            else {
                return false;
            };
            let Some(split) = pack.split_slot(idx) else {
                return false;
            };
            pack.grid[free] = Some(split);
            pack.sync_views();
            true
        }
    }
}

/// Deposits matching stackable items from the 24-slot pack grid (`6..30`) into chest `id`.
pub fn quick_stack_all(world: &mut ChestWorld, pack: &mut Pack, id: u64) -> bool {
    let Some(existing) = world.chests.find(id).map(|p| p.slots.clone()) else {
        return false;
    };
    let mut changed = false;
    for idx in BAR_SLOTS..TOTAL_SLOTS {
        let Some(entry @ SlotEntry::Stack { .. }) = pack.grid[idx] else {
            continue;
        };
        let Some(encoded) = codec::encode_slot(&entry, pack) else {
            continue;
        };
        let matches = existing
            .iter()
            .flatten()
            .any(|c| c.kind == ITEM_STACK && c.data == encoded.data);
        if matches && shift_transfer(world, pack, id, ChestCursor::Pack(idx)) {
            changed = true;
        }
    }
    changed
}
