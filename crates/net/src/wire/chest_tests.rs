//! Wire tests for protocol 7 chest, tree, dig, shot messages and `PlayerRecord.pack`.

use super::body_tests::planet;
use super::{
    ChestAct, ChestEvent, ClientMsg, DigOp, PlayerRecord, ServerMsg, Shot, TreeFell, decode, encode,
};
use crate::chest::{CHEST_SLOTS, ChestItem, ITEM_STACK, PlaceableKind, PlacedProp};
use crate::log::PlayerId;

/// Returns a sample placed chest with one item in slot 0.
fn sample_prop() -> PlacedProp {
    let mut slots = vec![None; CHEST_SLOTS];
    slots[0] = Some(ChestItem {
        kind: ITEM_STACK,
        count: 12,
        data: vec![1, 2, 3],
    });
    PlacedProp {
        id: 7,
        body: planet(),
        dir: [0.0, 1.0, 0.0],
        yaw: 0.75,
        kind: PlaceableKind::Chest,
        slots,
    }
}

/// Asserts that `message` survives length-prefixed postcard encode/decode unchanged.
fn check_wire<T>(message: T)
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let bytes = encode(&message).unwrap();
    let (got, used) = decode::<T>(&bytes, bytes.len()).unwrap().unwrap();
    assert_eq!((got, used), (message, bytes.len()));
}

/// Protocol 7 client chest messages survive length-prefixed postcard round-trips.
#[test]
fn protocol_7_client_chest_messages_come_back() {
    let (body, dir) = (planet(), [0.0, 1.0, 0.0]);
    let slot_item = sample_prop().slots[0].clone();
    check_wire(ClientMsg::Chest(ChestAct::Place {
        body,
        dir,
        yaw: 1.0,
        kind: PlaceableKind::Torch,
    }));
    check_wire(ClientMsg::Chest(ChestAct::Break { id: 7 }));
    check_wire(ClientMsg::Chest(ChestAct::SetSlot {
        id: 7,
        slot: 2,
        item: slot_item,
    }));
    check_wire(ClientMsg::Chest(ChestAct::Sort { id: 7 }));
}

/// Protocol 7 client tree, dig, and shot messages survive length-prefixed postcard round-trips.
#[test]
fn protocol_7_client_world_messages_come_back() {
    let (body, dir) = (planet(), [0.0, 1.0, 0.0]);
    check_wire(ClientMsg::Tree(TreeFell {
        body,
        id: "00000000000000ab".to_string(),
    }));
    check_wire(ClientMsg::Dig(DigOp {
        body,
        dir,
        radius: 1.85,
        depth: 0.48,
    }));
    check_wire(ClientMsg::Shot(Shot {
        body,
        origin: [1.0, 2.0, 3.0],
        dir: [0.0, 0.0, -1.0],
        kind: 1,
        barrel: 2,
        ammo: 3,
    }));
}

/// Protocol 7 chest server events survive length-prefixed postcard round-trips.
#[test]
fn protocol_7_chest_server_messages_come_back() {
    let prop = sample_prop();
    check_wire(ServerMsg::Chest(ChestEvent::Placed(prop.clone())));
    check_wire(ServerMsg::Chest(ChestEvent::Broken {
        id: 7,
        by: PlayerId(2),
    }));
    check_wire(ServerMsg::Chest(ChestEvent::Slot {
        id: 7,
        slot: 0,
        item: prop.slots[0].clone(),
    }));
    check_wire(ServerMsg::Chest(ChestEvent::Sorted {
        id: 7,
        slots: prop.slots.clone(),
    }));
    check_wire(ServerMsg::Chests(vec![prop]));
}

/// Protocol 7 tree, dig, and shot server messages survive length-prefixed postcard round-trips.
#[test]
fn protocol_7_world_server_messages_come_back() {
    let tree = TreeFell {
        body: planet(),
        id: "00000000000000cd".to_string(),
    };
    let dig = DigOp {
        body: planet(),
        dir: [0.6, 0.0, -0.8],
        radius: 1.5,
        depth: 0.4,
    };
    let shot = Shot {
        body: planet(),
        origin: [0.0, 10.0, 0.0],
        dir: [1.0, 0.0, 0.0],
        kind: 0,
        barrel: 1,
        ammo: 2,
    };
    check_wire(ServerMsg::Tree(tree.clone()));
    check_wire(ServerMsg::Trees(vec![tree]));
    check_wire(ServerMsg::Dig(dig));
    check_wire(ServerMsg::Digs(vec![dig]));
    check_wire(ServerMsg::Shot {
        by: PlayerId(4),
        shot,
    });
}

/// New client and server variants sit after existing tags, and `PlayerRecord.pack` defaults in TOML.
#[test]
fn protocol_7_variants_sit_at_the_end_and_pack_defaults() {
    let c = encode(&ClientMsg::Chest(ChestAct::Break { id: 1 })).unwrap();
    assert_eq!(c[4], 10);
    let s = encode(&ServerMsg::Chests(Vec::new())).unwrap();
    assert_eq!(s[4], 13);
    let old_toml = "place = \"p\"\nship = \"s\"\nmarks = \"m\"\n";
    let decoded: PlayerRecord = toml::from_str(old_toml).unwrap();
    assert_eq!(decoded.pack, "");
    let full = PlayerRecord {
        place: "p".to_string(),
        ship: "s".to_string(),
        marks: "m".to_string(),
        pack: "k".to_string(),
    };
    let bytes = encode(&ClientMsg::Record(full.clone())).unwrap();
    let (got, _) = decode::<ClientMsg>(&bytes, bytes.len()).unwrap().unwrap();
    assert_eq!(got, ClientMsg::Record(full));
}
