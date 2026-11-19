//! Reading the link once per frame: messages become Bevy messages, datagrams wait in the inbox.

use abstractions_net::io::{ClientEvent, CloseReason};
use abstractions_net::trust::short_key;
use abstractions_net::wire::ServerMsg;
use bevy::prelude::*;

use super::{Link, Role};

/// Most events read from the link in one frame. The rest wait for the next frame.
pub const EVENTS_PER_FRAME: usize = 256;

/// Most datagram batches kept in the inbox. Older ones are dropped first.
const INBOX_MAX: usize = 256;

/// The systems that read the link. Systems that read [`Inbox`] run after this set.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Receive;

/// The datagram batches that arrived this frame, oldest first, emptied at the start of each
/// read. Each batch holds state records of other players (see `abstractions_net::wire`).
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct Inbox(pub Vec<Vec<u8>>);

/// A message from the server, read this frame.
#[derive(Message, Debug, Clone, PartialEq)]
pub struct FromServer(pub ServerMsg);

/// The connection is over: nothing follows.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct Dropped(pub CloseReason);

/// The line a first sight of a server's key waits to show in the banner.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct FreshKey(pub Option<String>);

/// Where the events of a frame go.
type Outlets<'w> = (
    MessageWriter<'w, FromServer>,
    MessageWriter<'w, Dropped>,
    ResMut<'w, Inbox>,
);

/// Reads at most [`EVENTS_PER_FRAME`] events of the link. A server seen the first time
/// leaves its short key for the banner.
pub fn drain(
    mut link: ResMut<Link>,
    (mut messages, mut dropped, mut inbox): Outlets,
    mut fresh: ResMut<FreshKey>,
) {
    inbox.0.clear();
    let joining = matches!(link.role, Role::Join { .. });
    let Some(source) = link.source.as_mut() else {
        return;
    };
    for _ in 0..EVENTS_PER_FRAME {
        match source.poll() {
            Some(ClientEvent::Connected {
                fingerprint,
                first_time: true,
            }) if joining => {
                fresh.0 = Some(format!("New server key {}", short_key(&fingerprint)));
            }
            Some(ClientEvent::Connected { .. }) => {}
            Some(ClientEvent::Message(msg)) => {
                messages.write(FromServer(msg));
            }
            Some(ClientEvent::Datagram(batch)) => keep(&mut inbox, batch),
            Some(ClientEvent::Closed(reason)) => {
                dropped.write(Dropped(reason));
                return;
            }
            None => return,
        }
    }
}

/// Keeps `batch` in the inbox, dropping the oldest one when it is full.
fn keep(inbox: &mut Inbox, batch: Vec<u8>) {
    if inbox.0.len() >= INBOX_MAX {
        inbox.0.remove(0);
    }
    inbox.0.push(batch);
}
