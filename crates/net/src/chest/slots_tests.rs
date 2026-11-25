//! Unit tests of chest slot operations: swap/merge, split, quick-stack, and sort.

use super::tests::item;
use super::{CHEST_SLOTS, Chests, ITEM_GUN, ITEM_STACK, ITEM_TOOL, PlaceableKind};
use crate::ids::BodyId;

/// Swapping merges matching stacks up to the cap, swaps distinct ones, and never merges tools.
#[test]
fn swaps_and_merges_slots() {
    let mut chests = Chests::default();
    let id = chests
        .place(
            BodyId::default(),
            [0.0, 1.0, 0.0],
            0.0,
            PlaceableKind::Chest,
        )
        .unwrap()
        .id;
    assert!(!chests.swap_or_merge(id, 0, 1));
    assert!(!chests.swap_or_merge(id, 0, 0));
    chests.set_slot(id, 0, Some(item(ITEM_STACK, 40, 1)));
    chests.set_slot(id, 1, Some(item(ITEM_STACK, 50, 1)));
    assert!(chests.swap_or_merge(id, 0, 1));
    let prop = chests.find(id).unwrap();
    assert_eq!(prop.slots[1].as_ref().unwrap().count, 64);
    assert_eq!(prop.slots[0].as_ref().unwrap().count, 26);
    assert!(chests.swap_or_merge(id, 0, 4));
    assert!(chests.find(id).unwrap().slots[0].is_none());
    chests.set_slot(id, 2, Some(item(ITEM_TOOL, 1, 7)));
    chests.set_slot(id, 3, Some(item(ITEM_TOOL, 1, 7)));
    assert!(chests.swap_or_merge(id, 2, 3));
    assert_eq!(chests.find(id).unwrap().slots[2].as_ref().unwrap().count, 1);
}

/// Splitting takes half rounded up from a stack and takes whole single items.
#[test]
fn splits_stacks_and_single_items() {
    let mut chests = Chests::default();
    let id = chests
        .place(
            BodyId::default(),
            [0.0, 1.0, 0.0],
            0.0,
            PlaceableKind::Chest,
        )
        .unwrap()
        .id;
    chests.set_slot(id, 0, Some(item(ITEM_STACK, 5, 2)));
    let half = chests.split_slot(id, 0).unwrap();
    assert_eq!(half.count, 3);
    assert_eq!(chests.find(id).unwrap().slots[0].as_ref().unwrap().count, 2);
    chests.set_slot(id, 1, Some(item(ITEM_GUN, 1, 9)));
    let gun = chests.split_slot(id, 1).unwrap();
    assert_eq!(gun.kind, ITEM_GUN);
    assert!(chests.find(id).unwrap().slots[1].is_none());
    assert!(chests.split_slot(id, 1).is_none());
}

/// Quick-stacking fills partial stacks before empty slots and returns overflow when full.
#[test]
fn quick_stacks_into_partial_then_empty_slots() {
    let mut chests = Chests::default();
    let id = chests
        .place(
            BodyId::default(),
            [0.0, 1.0, 0.0],
            0.0,
            PlaceableKind::Chest,
        )
        .unwrap()
        .id;
    chests.set_slot(id, 2, Some(item(ITEM_STACK, 50, 3)));
    assert!(chests.quick_stack(id, item(ITEM_STACK, 20, 3)).is_none());
    let prop = chests.find(id).unwrap();
    assert_eq!(prop.slots[2].as_ref().unwrap().count, 64);
    assert_eq!(prop.slots[0].as_ref().unwrap().count, 6);
    for slot in 0..CHEST_SLOTS as u8 {
        chests.set_slot(id, slot, Some(item(ITEM_TOOL, 1, slot)));
    }
    chests.set_slot(id, 5, Some(item(ITEM_STACK, 60, 8)));
    let rem = chests.quick_stack(id, item(ITEM_STACK, 10, 8)).unwrap();
    assert_eq!(rem.count, 6);
    assert_eq!(
        chests.find(id).unwrap().slots[5].as_ref().unwrap().count,
        64
    );
}

/// Sorting merges scattered matching stacks and orders by kind, opaque data, and descending count.
#[test]
fn sorts_and_merges_chest_slots() {
    let mut chests = Chests::default();
    let id = chests
        .place(
            BodyId::default(),
            [0.0, 1.0, 0.0],
            0.0,
            PlaceableKind::Chest,
        )
        .unwrap()
        .id;
    chests.set_slot(id, 10, Some(item(ITEM_GUN, 1, 1)));
    chests.set_slot(id, 5, Some(item(ITEM_STACK, 40, 2)));
    chests.set_slot(id, 8, Some(item(ITEM_STACK, 40, 2)));
    chests.set_slot(id, 1, Some(item(ITEM_TOOL, 1, 4)));
    assert!(chests.sort_slots(id));
    let slots = &chests.find(id).unwrap().slots;
    assert_eq!(slots[0].as_ref().unwrap().count, 64);
    assert_eq!(slots[1].as_ref().unwrap().count, 16);
    assert_eq!(slots[2].as_ref().unwrap().kind, ITEM_TOOL);
    assert_eq!(slots[3].as_ref().unwrap().kind, ITEM_GUN);
    assert!(slots[4..].iter().all(Option::is_none));
    assert!(!chests.sort_slots(999));
}
