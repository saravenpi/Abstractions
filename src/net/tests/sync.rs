//! Tests of pack record persistence and chest, tree, and terrain dig synchronization.

use abstractions_net::chest::{CHEST_SLOTS, ChestItem, ITEM_STACK, PlaceableKind, PlacedProp};
use abstractions_net::io::ClientEvent;
use abstractions_net::log::{Changes, PlayerId};
use abstractions_net::wire::{ChestAct, ChestEvent, ClientMsg, DigOp, ServerMsg, TreeFell};
use bevy::prelude::*;

use super::kept::{kept, watch};
use super::rig::{self, push, run, scripted, sent, welcome};
use crate::craft::chest::ChestWorld;
use crate::craft::gun::{Gun, GunKind};
use crate::craft::matter::Mineral;
use crate::craft::pack::Pack;
use crate::net::join::world::{record_of, world_of};
use crate::saves::WorldSave;
use crate::universe::BodyId;
use crate::world::digs::{DigStamp, Digs, TerrainDug};
use crate::world::trees::{Felled, TREE_CHOPS, TreeChopped, TreeId};

/// Returns a headless app joined to a scripted server and its wire script.
pub(super) fn joined(tag: &str) -> (App, crate::net::link::Script) {
    let mut app = rig::app(tag);
    let script = scripted(&mut app);
    let hello = welcome(11, Changes::default(), None);
    push(&script, ClientEvent::Message(ServerMsg::Welcome(hello)));
    run(&mut app, 3);
    (app, script)
}

/// Returns a placed chest prop with `id` on `body` and empty storage slots.
fn sample_prop(id: u64, body: BodyId) -> PlacedProp {
    PlacedProp {
        id,
        body,
        dir: [0.0, 1.0, 0.0],
        yaw: 0.5,
        kind: PlaceableKind::Chest,
        slots: vec![None; CHEST_SLOTS],
    }
}

/// A player's pack serializes into `PlayerRecord.pack` and restores cleanly from a welcome.
#[test]
fn pack_record_round_trip() {
    let mut pack = Pack::default();
    pack.guns
        .push(Gun::new(1, GunKind::MineralRifle, Mineral::Ironstone));
    pack.next = 2;
    pack.kitted = true;
    let save = WorldSave {
        pack,
        ..WorldSave::default()
    };
    let record = record_of(&save);
    assert!(!record.pack.is_empty());
    let back = world_of(&welcome(8, Changes::default(), Some(record)), "Space");
    assert_eq!(back.save.pack, save.pack);
    let empty = world_of(&welcome(8, Changes::default(), None), "Space");
    assert_eq!(empty.save.pack, Pack::default());
}

/// Queued chest actions flush to the server as `ClientMsg::Chest`.
#[test]
fn chest_relay_sends_asks() {
    let (mut app, script) = joined("chest-asks");
    let act = ChestAct::Place {
        body: BodyId::default(),
        dir: [0.0, 1.0, 0.0],
        yaw: 0.25,
        kind: PlaceableKind::Chest,
    };
    app.world_mut()
        .resource_mut::<ChestWorld>()
        .asks
        .push(act.clone());
    run(&mut app, 2);
    assert!(sent(&script).contains(&ClientMsg::Chest(act)));
}

/// Server chest snapshots, slot updates, and break events update `ChestWorld`.
#[test]
fn chest_relay_applies_events() {
    let (mut app, script) = joined("chest-events");
    let item = ChestItem {
        kind: ITEM_STACK,
        count: 5,
        data: vec![1, 2, 3],
    };
    let slot_ev = ChestEvent::Slot {
        id: 7,
        slot: 2,
        item: Some(item.clone()),
    };
    let prop = sample_prop(7, BodyId::default());
    push(&script, ClientEvent::Message(ServerMsg::Chests(vec![prop])));
    push(&script, ClientEvent::Message(ServerMsg::Chest(slot_ev)));
    run(&mut app, 2);
    let found = &app
        .world()
        .resource::<ChestWorld>()
        .chests
        .find(7)
        .unwrap()
        .slots;
    assert_eq!(found[2], Some(item));
    let broken = ChestEvent::Broken {
        id: 7,
        by: PlayerId(1),
    };
    push(&script, ClientEvent::Message(ServerMsg::Chest(broken)));
    run(&mut app, 2);
    assert!(
        app.world()
            .resource::<ChestWorld>()
            .chests
            .find(7)
            .is_none()
    );
}

/// Felled trees send once per tree and remote tree falls update `Felled` without echoing back.
#[test]
fn tree_relay_avoids_echo_loop() {
    let (mut app, script) = joined("tree-relay");
    let (body, local_id, remote_id) = (BodyId::default(), TreeId(0x1234_5678), TreeId(0xDEAD_BEEF));
    app.world_mut().resource_mut::<Felled>().mark_gone(local_id);
    let chop = TreeChopped {
        planet: body,
        id: local_id,
        power: TREE_CHOPS,
    };
    app.world_mut().write_message(chop);
    run(&mut app, 2);
    app.world_mut().write_message(chop);
    run(&mut app, 2);
    let remote = TreeFell {
        body,
        id: remote_id.text(),
    };
    push(&script, ClientEvent::Message(ServerMsg::Tree(remote)));
    run(&mut app, 2);
    assert!(app.world().resource::<Felled>().gone(remote_id));
    let sent_trees: Vec<_> = sent(&script)
        .into_iter()
        .filter(|m| matches!(m, ClientMsg::Tree(_)))
        .collect();
    assert_eq!(
        sent_trees,
        vec![ClientMsg::Tree(TreeFell {
            body,
            id: local_id.text(),
        })]
    );
}

/// Local digs send to the server and remote digs update `Digs` and emit `TerrainDug` without echo.
#[test]
fn dig_relay_avoids_echo_loop() {
    let (mut app, script) = joined("dig-relay");
    watch::<TerrainDug>(&mut app);
    let body = BodyId::default();
    let local = DigStamp {
        planet: body,
        dir: [1.0, 0.0, 0.0],
        radius: 1.8,
        depth: 0.5,
    };
    app.world_mut().write_message(TerrainDug(local));
    run(&mut app, 2);
    let remote = DigOp {
        body,
        dir: [0.0, 1.0, 0.0],
        radius: 2.0,
        depth: 0.7,
    };
    push(&script, ClientEvent::Message(ServerMsg::Dig(remote)));
    run(&mut app, 2);
    assert_eq!(app.world().resource::<Digs>().all().len(), 1);
    assert_eq!(kept::<TerrainDug>(&app).len(), 2);
    let sent_digs = sent(&script)
        .into_iter()
        .filter(|m| matches!(m, ClientMsg::Dig(_)))
        .count();
    assert_eq!(sent_digs, 1);
}
