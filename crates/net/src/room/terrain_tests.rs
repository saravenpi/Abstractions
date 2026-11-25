//! Tests of room tree felling dedup, terrain digs, gunfire shots, and chunked join snapshots.

use super::Room;
use super::kit::{hello, join, messages};
use super::map_tests::pair;
use crate::chest::PlaceableKind;
use crate::host::{ConnId, Host, Outbox};
use crate::ids::BodyId;
use crate::log::PlayerId;
use crate::wire::{
    CHEST_CHUNK, ChestAct, ClientMsg, DIG_CHUNK, DigOp, ServerMsg, Shot, TREE_CHUNK, TREE_ID_MAX,
    TreeFell,
};

/// Seeds `room` from `ada` with multi-chunk chests, felled trees, and terrain digs.
fn seed_world_props(room: &mut Room, ada: ConnId, out: &mut Outbox) {
    let body = BodyId::default();
    for i in 0..(CHEST_CHUNK + 2) {
        let act = ChestAct::Place {
            body,
            dir: [0.0, 1.0, 0.0],
            yaw: i as f32 * 0.1,
            kind: PlaceableKind::Torch,
        };
        room.message(ada, ClientMsg::Chest(act), 101.0 + i as f64 * 0.02, out);
    }
    for i in 0..(TREE_CHUNK + 3) {
        let tree = TreeFell {
            body,
            id: format!("tree-{i}"),
        };
        room.message(ada, ClientMsg::Tree(tree), 103.0 + i as f64 * 0.05, out);
    }
    for i in 0..(DIG_CHUNK + 5) {
        let dig = DigOp {
            body,
            dir: [0.0, 1.0, 0.0],
            radius: 1.5,
            depth: 0.3 + i as f32 * 0.001,
        };
        room.message(ada, ClientMsg::Dig(dig), 110.0 + i as f64 * 0.05, out);
    }
}

/// Tree felling deduplicates by `(body, id)`, drops invalid ids, and relays to other clients.
#[test]
fn tree_felling_dedup_and_validation() {
    let (mut room, ada, bob, mut out) = pair();
    let tree = TreeFell {
        body: BodyId::default(),
        id: "0000000000000005".to_string(),
    };
    room.message(ada, ClientMsg::Tree(tree.clone()), 101.0, &mut out);
    room.message(bob, ClientMsg::Tree(tree.clone()), 101.1, &mut out);
    let bad = TreeFell {
        body: BodyId::default(),
        id: "x".repeat(TREE_ID_MAX + 1),
    };
    room.message(ada, ClientMsg::Tree(bad), 101.2, &mut out);
    assert_eq!(messages(&out, bob), [ServerMsg::Tree(tree.clone())]);
    assert!(messages(&out, ada).is_empty());
    assert_eq!(room.snapshot(102.0).trees, [tree]);
}

/// Terrain digs validate parameters, save to snapshots, and broadcast to other players.
#[test]
fn dig_broadcast_and_validation() {
    let (mut room, ada, bob, mut out) = pair();
    let dig = DigOp {
        body: BodyId::default(),
        dir: [0.0, 1.0, 0.0],
        radius: 1.85,
        depth: 0.48,
    };
    let bad = DigOp { depth: -1.0, ..dig };
    room.message(ada, ClientMsg::Dig(bad), 101.0, &mut out);
    room.message(ada, ClientMsg::Dig(dig), 101.1, &mut out);
    assert_eq!(messages(&out, bob), [ServerMsg::Dig(dig)]);
    assert!(messages(&out, ada).is_empty());
    assert_eq!(room.snapshot(102.0).digs, [dig]);
}

/// Gunfire shots relay ephemerally with sender player id and do not dirty or save to snapshots.
#[test]
fn shot_relay_is_ephemeral_and_stamped() {
    let (mut room, ada, bob, mut out) = pair();
    room.saved();
    let shot = Shot {
        body: BodyId::default(),
        origin: [1.0, 2.0, 3.0],
        dir: [0.0, 0.0, -1.0],
        kind: 1,
        barrel: 2,
        ammo: 0,
    };
    let bad = Shot {
        origin: [f32::NAN, 0.0, 0.0],
        ..shot
    };
    room.message(ada, ClientMsg::Shot(bad), 101.0, &mut out);
    room.message(ada, ClientMsg::Shot(shot), 101.1, &mut out);
    let sent = ServerMsg::Shot {
        by: PlayerId(1),
        shot,
    };
    assert_eq!(messages(&out, bob), [sent]);
    assert!(messages(&out, ada).is_empty());
    assert!(!room.changed());
}

/// Joining players receive placed props, felled trees, and digs split into protocol chunks.
#[test]
fn join_receives_chunked_snapshots() {
    let (mut room, ada, _, mut out) = pair();
    seed_world_props(&mut room, ada, &mut out);
    let mut cyd_out = Outbox::default();
    let cyd = join(&mut room, 3, hello("Cyd", 3), 120.0, &mut cyd_out);
    let msgs = messages(&cyd_out, cyd);
    let chests: Vec<_> = msgs
        .iter()
        .filter_map(|m| match m {
            ServerMsg::Chests(c) => Some(c.len()),
            _ => None,
        })
        .collect();
    let trees: Vec<_> = msgs
        .iter()
        .filter_map(|m| match m {
            ServerMsg::Trees(t) => Some(t.len()),
            _ => None,
        })
        .collect();
    let digs: Vec<_> = msgs
        .iter()
        .filter_map(|m| match m {
            ServerMsg::Digs(d) => Some(d.len()),
            _ => None,
        })
        .collect();
    assert_eq!(chests, [CHEST_CHUNK, 2]);
    assert_eq!(trees, [TREE_CHUNK, 3]);
    assert_eq!(digs, [DIG_CHUNK, 5]);
}

/// Restarting a room from a snapshot serves saved chests, trees, and digs to new joiners.
#[test]
fn restart_restores_chests_trees_and_digs() {
    let (mut room, ada, _, mut out) = pair();
    seed_world_props(&mut room, ada, &mut out);
    let cfg = super::RoomConfig {
        name: "R".to_string(),
        password: None,
        cap: 4,
        version: "0.13.0".to_string(),
    };
    let mut restarted = Room::new(cfg, room.snapshot(120.0), 0.0);
    let mut dan_out = Outbox::default();
    let dan = join(&mut restarted, 4, hello("Dan", 4), 1.0, &mut dan_out);
    let dan_msgs = messages(&dan_out, dan);
    assert!(dan_msgs.iter().any(|m| matches!(m, ServerMsg::Chests(_))));
    assert!(dan_msgs.iter().any(|m| matches!(m, ServerMsg::Trees(_))));
    assert!(dan_msgs.iter().any(|m| matches!(m, ServerMsg::Digs(_))));
}
