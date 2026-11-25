//! Placed chests and torches in a session: the server is authoritative. Local placement, break,
//! slot, and sort requests go to the server, and server snapshots and events update [`ChestWorld`].

use abstractions_net::chest::Chests;
use abstractions_net::wire::{ChestEvent, ClientMsg, ServerMsg};
use bevy::prelude::*;

use super::link::{FromServer, Link, Receive, Role};
use crate::craft::chest::ChestWorld;

/// Hands placed chests over to the server when a link opens. A joined world starts empty until the
/// server sends its snapshot; a host keeps its own props, which its server started from.
fn hand_over(link: Res<Link>, mut chests: ResMut<ChestWorld>) {
    chests.remote = true;
    chests.asks.clear();
    if matches!(link.role, Role::Join { .. }) {
        chests.chests = Chests::default();
    }
}

/// Run condition: local chest actions are queued to send to the server.
fn asking(chests: Option<Res<ChestWorld>>) -> bool {
    chests.is_some_and(|chests| !chests.asks.is_empty())
}

/// Flushes queued local chest actions to the server, oldest first.
fn send(mut link: ResMut<Link>, mut chests: ResMut<ChestWorld>) {
    for act in std::mem::take(&mut chests.asks) {
        link.send(ClientMsg::Chest(act));
    }
}

/// Applies one server [`ChestEvent`] to `chests`.
pub(crate) fn apply_event(chests: &mut Chests, event: ChestEvent) {
    match event {
        ChestEvent::Placed(prop) => chests.told(prop),
        ChestEvent::Broken { id, .. } => chests.forget(id),
        ChestEvent::Slot { id, slot, item } => {
            if let Some(prop) = chests.find_mut(id)
                && let Some(cell) = prop.slots.get_mut(usize::from(slot))
            {
                *cell = item;
            }
        }
        ChestEvent::Sorted { id, slots } => {
            if let Some(prop) = chests.find_mut(id) {
                prop.slots = slots;
            }
        }
    }
}

/// Applies placed chest snapshots and live events received from the server this frame.
fn receive(mut from: MessageReader<FromServer>, mut chests: ResMut<ChestWorld>) {
    for FromServer(msg) in from.read() {
        match msg {
            ServerMsg::Chests(batch) => {
                for prop in batch {
                    chests.chests.told(prop.clone());
                }
            }
            ServerMsg::Chest(ev) => apply_event(&mut chests.chests, ev.clone()),
            _ => {}
        }
    }
}

/// Restores local chest authority once the session link closes.
fn take_back(mut chests: ResMut<ChestWorld>) {
    chests.remote = false;
    chests.asks.clear();
}

/// Registers chest synchronization for networked sessions. Every system waits for [`Link`], so
/// single player runs none.
pub(super) fn plugin(app: &mut App) {
    let linked = resource_exists::<Link>;
    app.init_resource::<ChestWorld>();
    app.add_systems(
        Update,
        (
            hand_over.run_if(resource_added::<Link>),
            receive
                .after(Receive)
                .run_if(linked)
                .run_if(on_message::<FromServer>),
            send.run_if(linked).run_if(asking),
            take_back.run_if(resource_removed::<Link>),
        )
            .chain(),
    );
}
