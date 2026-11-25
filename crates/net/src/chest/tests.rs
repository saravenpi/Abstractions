//! Unit tests of chest validation, placement, slot setting, and client sync helpers.

use super::{
    CHEST_ITEM_BYTES, CHEST_MAX_PROPS, CHEST_SLOTS, ChestItem, Chests, ITEM_GUN, ITEM_STACK,
    ITEM_TOOL, PlaceableKind, PlacedProp, STACK_MAX, valid_dir, valid_item, valid_prop,
};
use crate::ids::BodyId;

/// Returns a chest item of `kind` with `count` and payload tagged by `tag`.
pub(super) fn item(kind: u8, count: u16, tag: u8) -> ChestItem {
    ChestItem {
        kind,
        count,
        data: vec![tag; 4],
    }
}

/// Direction and item validation checks bounds, finiteness, and kind-specific counts.
#[test]
fn validates_dirs_and_items() {
    assert!(valid_dir([0.0, 1.0, 0.0]));
    assert!(!valid_dir([0.0, 0.0, 0.0]));
    assert!(!valid_dir([f32::NAN, 1.0, 0.0]));
    assert!(!valid_dir([f32::MAX, f32::MAX, 0.0]));
    assert!(valid_item(&item(ITEM_STACK, STACK_MAX, 1)));
    assert!(!valid_item(&item(ITEM_STACK, 0, 1)));
    assert!(!valid_item(&item(ITEM_STACK, STACK_MAX + 1, 1)));
    assert!(valid_item(&item(ITEM_TOOL, 1, 2)));
    assert!(!valid_item(&item(ITEM_TOOL, 2, 2)));
    assert!(!valid_item(&item(ITEM_GUN, 2, 3)));
    assert!(!valid_item(&item(9, 1, 1)));
    let big = ChestItem {
        kind: ITEM_STACK,
        count: 1,
        data: vec![0; CHEST_ITEM_BYTES + 1],
    };
    assert!(!valid_item(&big));
}

/// Prop validation checks finite yaw, direction, and chest vs torch slot count.
#[test]
fn validates_props_and_serde() {
    let mut chest = PlacedProp {
        id: 1,
        body: BodyId::default(),
        dir: [0.0, 1.0, 0.0],
        yaw: 0.0,
        kind: PlaceableKind::Chest,
        slots: vec![None; CHEST_SLOTS],
    };
    assert!(valid_prop(&chest));
    chest.yaw = f32::INFINITY;
    assert!(!valid_prop(&chest));
    chest.yaw = 0.0;
    chest.slots.pop();
    assert!(!valid_prop(&chest));
    let torch = PlacedProp {
        kind: PlaceableKind::Torch,
        slots: Vec::new(),
        ..chest
    };
    assert!(valid_prop(&torch));
}

/// Placing assigns increasing ids, normalizes directions, and removes cleanly.
#[test]
fn places_finds_and_removes_props() {
    let mut chests = Chests::default();
    let body = BodyId::default();
    assert!(
        chests
            .place(body, [0.0; 3], 0.0, PlaceableKind::Chest)
            .is_none()
    );
    let c = chests
        .place(body, [0.0, 2.0, 0.0], 0.5, PlaceableKind::Chest)
        .unwrap();
    assert_eq!(
        (c.id, c.dir, c.slots.len()),
        (1, [0.0, 1.0, 0.0], CHEST_SLOTS)
    );
    let t = chests
        .place(body, [1.0, 0.0, 0.0], 0.0, PlaceableKind::Torch)
        .unwrap();
    assert_eq!((t.id, t.slots.len()), (2, 0));
    assert_eq!(chests.find(1).map(|p| p.id), Some(1));
    assert_eq!(chests.remove(1).map(|p| p.id), Some(1));
    assert!(chests.find(1).is_none());
}

/// Placement enforces `CHEST_MAX_PROPS` and frees space after removal.
#[test]
fn enforces_prop_cap() {
    let mut chests = Chests::default();
    let body = BodyId::default();
    for _ in 0..CHEST_MAX_PROPS {
        assert!(
            chests
                .place(body, [0.0, 1.0, 0.0], 0.0, PlaceableKind::Torch)
                .is_some()
        );
    }
    assert!(
        chests
            .place(body, [0.0, 1.0, 0.0], 0.0, PlaceableKind::Torch)
            .is_none()
    );
    assert!(chests.remove(1).is_some());
    assert!(
        chests
            .place(body, [0.0, 1.0, 0.0], 0.0, PlaceableKind::Torch)
            .is_some()
    );
}

/// Setting slots validates items and bounds; told/forget keep client state synchronized.
#[test]
fn sets_slots_and_syncs_client_copies() {
    let mut chests = Chests::default();
    let b = BodyId::default();
    let chest = chests
        .place(b, [0.0, 1.0, 0.0], 0.0, PlaceableKind::Chest)
        .unwrap();
    let torch = chests
        .place(b, [1.0, 0.0, 0.0], 0.0, PlaceableKind::Torch)
        .unwrap();
    assert!(chests.set_slot(chest.id, 3, Some(item(ITEM_STACK, 10, 5))));
    assert!(!chests.set_slot(chest.id, CHEST_SLOTS as u8, Some(item(ITEM_STACK, 1, 1))));
    assert!(!chests.set_slot(chest.id, 0, Some(item(ITEM_STACK, 99, 1))));
    assert!(!chests.set_slot(torch.id, 0, Some(item(ITEM_STACK, 1, 1))));
    let mut mirror = Chests::new(chests.props.clone());
    assert_eq!(mirror.next_id, 3);
    let updated = PlacedProp {
        yaw: 1.25,
        ..chests.find(chest.id).unwrap().clone()
    };
    mirror.told(updated);
    assert_eq!(mirror.find(chest.id).unwrap().yaw, 1.25);
    mirror.forget(torch.id);
    assert!(mirror.find(torch.id).is_none());
}
