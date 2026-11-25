//! Tests of room chest placement, slot updates, sorting, breaking, and `PlayerRecord.pack`.

use super::Room;
use super::kit::{hello, join, messages, place_text, room};
use super::map_tests::pair;
use crate::chest::{ChestItem, ITEM_STACK, PlaceableKind};
use crate::host::{ConnId, Host, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{ChestAct, ChestEvent, ClientMsg, PlayerRecord, ServerMsg};

/// Returns a stackable chest item tagged by `tag` with `count`.
fn stack(tag: u8, count: u16) -> ChestItem {
    ChestItem {
        kind: ITEM_STACK,
        count,
        data: vec![tag],
    }
}

/// Places a chest from `conn` on the default body.
fn place_chest(room: &mut Room, conn: ConnId, out: &mut Outbox) {
    let place = ChestAct::Place {
        body: BodyId::default(),
        dir: [0.0, 1.0, 0.0],
        yaw: 0.5,
        kind: PlaceableKind::Chest,
    };
    room.message(conn, ClientMsg::Chest(place), 101.0, out);
}

/// Placing and slot setting synchronize across two clients.
#[test]
fn chest_place_and_slot_sync() {
    let (mut room, ada, bob, mut out) = pair();
    place_chest(&mut room, ada, &mut out);
    assert!(matches!(
        messages(&out, ada).as_slice(),
        [ServerMsg::Chest(ChestEvent::Placed(p))] if p.id == 1
    ));
    assert_eq!(messages(&out, ada), messages(&out, bob));
    out.clear();
    let set = ClientMsg::Chest(ChestAct::SetSlot {
        id: 1,
        slot: 4,
        item: Some(stack(9, 30)),
    });
    room.message(ada, set, 101.1, &mut out);
    assert!(matches!(
        messages(&out, bob).as_slice(),
        [ServerMsg::Chest(ChestEvent::Slot { id: 1, slot: 4, item: Some(i) })] if i.count == 30
    ));
}

/// Sorting merges chest stacks across clients.
#[test]
fn chest_sort_syncs() {
    let (mut room, ada, bob, mut out) = pair();
    place_chest(&mut room, ada, &mut out);
    let set = |slot, count| {
        ClientMsg::Chest(ChestAct::SetSlot {
            id: 1,
            slot,
            item: Some(stack(9, count)),
        })
    };
    room.message(ada, set(4, 30), 101.1, &mut out);
    room.message(ada, set(2, 40), 101.2, &mut out);
    out.clear();
    room.message(
        ada,
        ClientMsg::Chest(ChestAct::Sort { id: 1 }),
        101.3,
        &mut out,
    );
    assert!(matches!(
        messages(&out, bob).as_slice(),
        [ServerMsg::Chest(ChestEvent::Sorted { id: 1, slots })]
            if slots[0].as_ref().unwrap().count == 64 && slots[1].as_ref().unwrap().count == 6
    ));
}

/// Breaking a chest broadcasts removal with breaker id and clears it from snapshots.
#[test]
fn chest_break_syncs() {
    let (mut room, ada, bob, mut out) = pair();
    place_chest(&mut room, ada, &mut out);
    out.clear();
    room.message(
        bob,
        ClientMsg::Chest(ChestAct::Break { id: 1 }),
        102.0,
        &mut out,
    );
    let broken = ServerMsg::Chest(ChestEvent::Broken {
        id: 1,
        by: PlayerId(2),
    });
    assert_eq!(messages(&out, ada), std::slice::from_ref(&broken));
    assert_eq!(messages(&out, bob), [broken]);
    assert!(room.snapshot(103.0).chests.props.is_empty());
}

/// `PlayerRecord.pack` round-trips across reconnects and an empty `pack` preserves the kept one.
#[test]
fn player_record_pack_round_trips_and_keeps_nonempty() {
    let mut room = room(4, None);
    let mut out = Outbox::default();
    let ada = join(&mut room, 1, hello("Ada", 1), 100.0, &mut out);
    let saved_pack = PlayerRecord {
        place: place_text("2.0"),
        ship: String::new(),
        marks: String::new(),
        pack: "kitted = true\n".to_string(),
    };
    room.message(ada, ClientMsg::Record(saved_pack.clone()), 101.0, &mut out);
    let empty_pack = PlayerRecord {
        pack: String::new(),
        ..saved_pack
    };
    room.message(ada, ClientMsg::Record(empty_pack), 102.0, &mut out);
    room.message(ada, ClientMsg::Bye, 103.0, &mut out);
    out.clear();
    let back = join(&mut room, 2, hello("Ada", 1), 104.0, &mut out);
    let welcome = messages(&out, back)
        .into_iter()
        .find_map(|m| match m {
            ServerMsg::Welcome(w) => Some(w),
            _ => None,
        })
        .unwrap();
    assert_eq!(welcome.record.unwrap().pack, "kitted = true\n");
}
