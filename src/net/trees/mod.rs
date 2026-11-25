//! Felled tree synchronization in a session: local tree falls go to the server once, and server
//! snapshots and remote falls mark trees gone in [`Felled`] without echoing back.

use std::collections::HashSet;

use abstractions_net::wire::{ClientMsg, ServerMsg, TreeFell};
use bevy::prelude::*;

use super::link::{FromServer, Link, Receive, Role};
use crate::world::trees::{Felled, TREE_CHOPS, TreeChopped, TreeId};

/// Tracks felled tree ids already sent to or received from the server to prevent echo loops.
#[derive(Resource, Debug, Default)]
pub struct TreeSync {
    /// Tree ids already synced during the current session.
    seen: HashSet<TreeId>,
}

/// Resets tree sync state when a link opens; joined clients start from the server snapshot.
fn reset(link: Res<Link>, mut sync: ResMut<TreeSync>, mut felled: ResMut<Felled>) {
    sync.seen.clear();
    if matches!(link.role, Role::Join { .. }) {
        *felled = Felled::default();
    } else {
        sync.seen.extend(felled.felled_ids());
    }
}

/// Records one server-reported felled tree into `felled` and notifies loaded chunks.
fn apply_fell(
    fell: &TreeFell,
    sync: &mut TreeSync,
    felled: &mut Felled,
    chopped: &mut MessageWriter<TreeChopped>,
) {
    let Some(id) = TreeId::parse(&fell.id) else {
        return;
    };
    sync.seen.insert(id);
    felled.mark_gone(id);
    chopped.write(TreeChopped {
        planet: fell.body,
        id,
        power: TREE_CHOPS,
    });
}

/// Applies felled tree snapshots and live falls received from the server this frame.
fn receive(
    mut from: MessageReader<FromServer>,
    mut sync: ResMut<TreeSync>,
    mut felled: ResMut<Felled>,
    mut chopped: MessageWriter<TreeChopped>,
) {
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Trees(batch) => {
                for fell in batch {
                    apply_fell(fell, &mut sync, &mut felled, &mut chopped);
                }
            }
            ServerMsg::Tree(fell) => apply_fell(fell, &mut sync, &mut felled, &mut chopped),
            _ => {}
        }
    }
}

/// Sends newly felled local trees to the server once each.
fn send(
    mut link: ResMut<Link>,
    mut sync: ResMut<TreeSync>,
    felled: Res<Felled>,
    mut chopped: MessageReader<TreeChopped>,
) {
    for chop in chopped.read() {
        if felled.gone(chop.id) && sync.seen.insert(chop.id) {
            link.send(ClientMsg::Tree(TreeFell {
                body: chop.planet,
                id: chop.id.text(),
            }));
        }
    }
}

/// Registers felled tree synchronization for networked sessions. Every system waits for [`Link`],
/// so single player runs none.
pub(super) fn plugin(app: &mut App) {
    let linked = resource_exists::<Link>;
    app.add_message::<TreeChopped>()
        .init_resource::<Felled>()
        .init_resource::<TreeSync>()
        .add_systems(
            Update,
            (
                reset.run_if(resource_added::<Link>),
                receive
                    .after(Receive)
                    .run_if(linked)
                    .run_if(on_message::<FromServer>),
                send.run_if(linked).run_if(on_message::<TreeChopped>),
            )
                .chain(),
        );
}
