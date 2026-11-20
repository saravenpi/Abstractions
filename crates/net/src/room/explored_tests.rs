//! Tests of the explored ground the room shares: leaves merged and sent once a second.

use super::kit::messages;
use super::map_tests::{pair, welcome_of_cyd};
use crate::host::Host;
use crate::ids::BodyId;
use crate::leaves::{Leaves, Run, key};
use crate::wire::{ClientMsg, ServerMsg};

/// Returns the leaves of every explored message in `msgs`, in order.
fn leaves(msgs: &[ServerMsg]) -> Vec<Vec<u64>> {
    let explored = msgs.iter().filter_map(|msg| match msg {
        ServerMsg::Explored { runs, .. } => Some(listed(&Leaves::from_runs(runs))),
        _ => None,
    });
    explored.collect()
}

/// Returns the keys of `leaves` counted from `key(1, 6, 0)`.
fn listed(leaves: &Leaves) -> Vec<u64> {
    (0..4096)
        .filter(|n| leaves.contains(key(1, 6, *n)))
        .collect()
}

/// Returns the runs of the leaves of face 1, depth 6, with these Morton codes.
fn runs_of(codes: &[u64]) -> Vec<Run> {
    let mut leaves = Leaves::default();
    for code in codes {
        leaves.insert(key(1, 6, *code));
    }
    leaves.runs()
}

/// New leaves go to everyone at most once a second per body, and known or bad ones never do.
#[test]
fn explored_leaves_merge_and_go_out_once_a_second() {
    let (mut room, ada, bob, mut out) = pair();
    let body = BodyId::default();
    let sent = |runs: Vec<Run>| ClientMsg::Explored { body, runs };
    let mut first = runs_of(&[3, 70]);
    first.push(Run {
        gap: key(7, 6, 0),
        len: 2,
    });
    room.message(ada, sent(first), 100.0, &mut out);
    room.tick(100.0, &mut out);
    assert_eq!(leaves(&messages(&out, bob)), [vec![3, 70]]);
    out.clear();
    room.message(bob, sent(runs_of(&[70, 5])), 100.2, &mut out);
    room.tick(100.5, &mut out);
    assert!(leaves(&messages(&out, ada)).is_empty());
    room.tick(101.0, &mut out);
    assert_eq!(leaves(&messages(&out, ada)), [vec![5]]);
}

/// A later player finds every explored leaf in the welcome, and the snapshot holds them.
#[test]
fn a_later_player_finds_the_explored_ground() {
    let (mut room, ada, _, mut out) = pair();
    let body = BodyId::default();
    let runs = runs_of(&[3, 5, 70]);
    room.message(ada, ClientMsg::Explored { body, runs }, 100.0, &mut out);
    let welcome = welcome_of_cyd(&mut room, 100.5);
    let (got, runs) = &welcome.explored[0];
    assert_eq!(*got, body);
    assert_eq!(listed(&Leaves::from_runs(runs)), [3, 5, 70]);
    assert_eq!(room.snapshot(102.0).explored, welcome.explored);
}
