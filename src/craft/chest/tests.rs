//! Unit tests for placed chests, torches, codec round-trips, dual-grid UI ops, break spill,
//! and world save persistence.

use abstractions_net::chest::PlaceableKind as NetKind;
use abstractions_net::wire::ChestAct;

use super::place::{break_placed, place_prop};
use super::{ChestCursor, ChestWorld, OpenChest, codec, ui_ops};
use crate::craft::build::{Intent, solve};
use crate::craft::ground::Shed;
use crate::craft::gun::GunKind;
use crate::craft::matter::{Mineral, PlaceableKind, Stuff, WoodKind};
use crate::craft::pack::{BAR_SLOTS, Pack, SlotEntry};
use crate::craft::stage::{earth, pack};
use crate::saves::WorldSave;
use crate::universe::BodyId;

/// Places a local chest and a remote torch, verifying pack consumption and ask queueing.
#[test]
fn place_chest_and_torch() {
    let mut inv = Pack::default();
    inv.grid[0] = Some(SlotEntry::Stack {
        stuff: Stuff::Placeable(PlaceableKind::Chest),
        material: PlaceableKind::Chest.material(),
        count: 2,
    });
    inv.grid[1] = Some(SlotEntry::Stack {
        stuff: Stuff::Placeable(PlaceableKind::Torch),
        material: PlaceableKind::Torch.material(),
        count: 1,
    });
    inv.sync_views();
    let mut world = ChestWorld::default();
    assert_eq!(
        place_prop(
            &mut inv,
            &mut world,
            BodyId::default(),
            [0.0, 1.0, 0.0],
            0.5
        ),
        Some(PlaceableKind::Chest)
    );
    assert_eq!(
        (world.chests.props.len(), world.chests.props[0].kind),
        (1, NetKind::Chest)
    );
    assert_eq!(inv.count_of(&Stuff::Placeable(PlaceableKind::Chest)), 1);
    world.remote = true;
    inv.bar.held = 1;
    assert_eq!(
        place_prop(
            &mut inv,
            &mut world,
            BodyId::default(),
            [1.0, 0.0, 0.0],
            1.2
        ),
        Some(PlaceableKind::Torch)
    );
    assert!(matches!(
        world.asks.as_slice(),
        [ChestAct::Place {
            kind: NetKind::Torch,
            ..
        }]
    ));
}

/// Round-trips stack, tool, and firearm slots through [`codec`] and validates 3D previews.
#[test]
fn codec_round_trip() {
    let mut src = pack();
    let stock = src.stock();
    let props = solve(Intent::Cut, &stock, &earth()).expect("cut proposals");
    let tool_id = src.make(&props.lightest, &stock).expect("tool crafted");
    let gun_id = src
        .keep_gun(GunKind::FlintlockPistol, Mineral::Flint)
        .expect("gun crafted");
    let entries = [
        SlotEntry::Stack {
            stuff: Stuff::Wood(WoodKind::Plank),
            material: WoodKind::Plank.material(),
            count: 12,
        },
        SlotEntry::Tool(tool_id),
        SlotEntry::Gun(gun_id),
    ];
    let mut dst = Pack::default();
    for entry in entries {
        let encoded = codec::encode_slot(&entry, &src).expect("encoded");
        assert!(codec::preview(&encoded).is_some());
        let decoded = codec::decode_to_pack(&encoded, &mut dst).expect("decoded");
        assert_eq!(
            encoded,
            codec::encode_slot(&decoded, &dst).expect("re-encoded")
        );
    }
}

/// Exercises Shift-transfer, left-click merge, right-click split, quick-stack, and sort.
#[test]
fn transfer_split_quick_stack_sort() {
    let mut world = ChestWorld::default();
    let id = world
        .chests
        .place(BodyId::default(), [0.0, 1.0, 0.0], 0.0, NetKind::Chest)
        .expect("chest")
        .id;
    let mut inv = Pack::default();
    let plank = Stuff::Wood(WoodKind::Plank);
    inv.add_stack(plank, WoodKind::Plank.material(), 40);
    let mut open = OpenChest {
        id: Some(id),
        selected: None,
    };
    assert!(ui_ops::click_slot(
        &mut world,
        &mut inv,
        &mut open,
        ChestCursor::Pack(BAR_SLOTS),
        true
    ));
    assert_eq!(inv.count_of(&plank), 0);
    assert!(ui_ops::split_slot(
        &mut world,
        &mut inv,
        &mut open,
        ChestCursor::Chest(0)
    ));
    let prop = world.chests.find(id).expect("chest");
    assert_eq!(
        (
            prop.slots[0].as_ref().map(|i| i.count),
            prop.slots[1].as_ref().map(|i| i.count)
        ),
        (Some(20), Some(20))
    );
    inv.add_stack(plank, WoodKind::Plank.material(), 30);
    assert!(ui_ops::quick_stack_all(&mut world, &mut inv, id));
    assert_eq!(inv.count_of(&plank), 0);
    world.emit(ChestAct::Sort { id });
    let sorted = world.chests.find(id).expect("chest");
    assert_eq!(
        (
            sorted.slots[0].as_ref().map(|i| i.count),
            sorted.slots[1].as_ref().map(|i| i.count)
        ),
        (Some(64), Some(6))
    );
}

/// Breaking a chest spills every stored item plus the chest placeable into [`Shed`].
#[test]
fn break_spills_contents_and_prop() {
    let mut world = ChestWorld::default();
    let id = world
        .chests
        .place(BodyId::default(), [0.0, 1.0, 0.0], 0.0, NetKind::Chest)
        .expect("chest")
        .id;
    let inv = Pack::default();
    let entry = SlotEntry::Stack {
        stuff: Stuff::Wood(WoodKind::Charcoal),
        material: WoodKind::Charcoal.material(),
        count: 5,
    };
    let item = codec::encode_slot(&entry, &inv).expect("encoded");
    assert!(world.chests.set_slot(id, 0, Some(item)));
    let mut shed = Shed::default();
    assert!(break_placed(id, &mut world, &mut shed));
    assert!(world.chests.find(id).is_none());
    assert_eq!(shed.0.len(), 2);
}

/// [`WorldSave`] with placed chests and torches round-trips through TOML unchanged.
#[test]
fn world_save_round_trip() {
    let mut save = WorldSave::default();
    let chest = save
        .chests
        .place(BodyId::default(), [0.0, 1.0, 0.0], 0.75, NetKind::Chest)
        .expect("chest");
    let _ = save
        .chests
        .place(BodyId::default(), [1.0, 0.0, 0.0], 1.5, NetKind::Torch)
        .expect("torch");
    let inv = Pack::default();
    let entry = SlotEntry::Stack {
        stuff: Stuff::Mineral(Mineral::Copper),
        material: Mineral::Copper.material(),
        count: 8,
    };
    let item = codec::encode_slot(&entry, &inv).expect("encoded");
    assert!(save.chests.set_slot(chest.id, 3, Some(item)));
    let text = toml::to_string(&save).expect("serialize");
    let back: WorldSave = toml::from_str(&text).expect("deserialize");
    assert_eq!(back.chests, save.chests);
}
