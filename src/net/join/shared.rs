//! The log every player of a server shares: the local first visits go to the server, and what
//! the server logs comes back into the log and the atlas of every player.

use abstractions_net::wire::{ClientMsg, ServerMsg};
use bevy::prelude::*;

use crate::computer::{Atlas, Target};
use crate::net::link::{FromServer, Link};
use crate::sim::{Change, Changes, ShareChange};

/// Sends the changes the local player made to the server, which logs them for everyone.
pub fn share(mut shares: MessageReader<ShareChange>, mut link: ResMut<Link>) {
    for ShareChange(change) in shares.read() {
        link.send(ClientMsg::Change(change.clone()));
    }
}

/// Appends each change the server logged to the log, and adds a visited body to the atlas
/// when it is missing, so the board computer shows the planets other players found.
pub fn take_changes(
    mut from: MessageReader<FromServer>,
    changes: Option<ResMut<Changes>>,
    atlas: Option<ResMut<Atlas>>,
) {
    let (Some(mut changes), mut atlas) = (changes, atlas) else {
        return;
    };
    for FromServer(msg) in from.read() {
        let ServerMsg::Change(entry) = msg else {
            continue;
        };
        if changes.entries.iter().all(|old| old.change != entry.change) {
            changes.entries.push(entry.clone());
        }
        let Change::Visited(body) = entry.change;
        if let Some(atlas) = atlas.as_deref_mut() {
            atlas.visit(Target::Body(body));
            atlas.visit(Target::Star(body.star));
            atlas.visit(Target::Galaxy(body.star.galaxy));
        }
    }
}
