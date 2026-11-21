//! The ground in a session: the server keeps it. What this game puts down or takes is asked of the
//! server, and what the server tells (what lies, what left, what was granted or handed back) comes
//! back into [`Ground`], the pack and the banner.

#[cfg(test)]
mod tests;

use abstractions_net::ground::Pile;
use abstractions_net::wire::{ClientMsg, Event, ServerMsg};
use bevy::prelude::*;

use super::link::{FromServer, Link, Receive, Role};
use crate::craft::ground::{Ground, Note, Shed};
use crate::craft::{Burden, Pack};
use crate::sim::PlayerId;

/// Returns true for an event about the things on the ground, which this module takes.
pub(super) fn grounded(event: &Event) -> bool {
    matches!(
        event,
        Event::Lies(_)
            | Event::Gone { .. }
            | Event::Granted(_)
            | Event::Denied(_)
            | Event::Kept(_)
            | Event::Ground(_)
    )
}

/// Turns the ground over to the server when a link opens. A joined world starts with an empty
/// ground that the server tells; a host keeps its own, which its server started from.
fn hand_over(link: Res<Link>, mut ground: ResMut<Ground>) {
    (ground.remote, ground.asked) = (true, None);
    ground.asks.clear();
    if matches!(link.role, Role::Join { .. }) {
        ground.pile = Pile::default();
    }
}

/// Run condition: the ground has something to ask the server.
fn asking(ground: Option<Res<Ground>>) -> bool {
    ground.is_some_and(|ground| !ground.asks.is_empty())
}

/// Sends what the ground asks of the server, oldest first.
fn send(mut link: ResMut<Link>, mut ground: ResMut<Ground>) {
    for act in std::mem::take(&mut ground.asks) {
        link.send(ClientMsg::Hunt(act));
    }
}

/// What an answer of the server changes: the ground, the pack, what waits to be put down, and the
/// room left in the pack.
pub(super) type Answer<'a> = (&'a mut Ground, &'a mut Pack, &'a mut Shed, f32);

/// Applies one event of the ground for the player `you`, and returns the line for the banner.
pub(super) fn hear(
    event: Event,
    you: Option<PlayerId>,
    (ground, pack, shed, room): Answer,
) -> Option<&'static str> {
    match event {
        Event::Lies(lying) => ground.heard(lying, you),
        Event::Ground(all) => all.into_iter().for_each(|lying| ground.pile.told(lying)),
        Event::Gone { id, .. } => ground.pile.forget(id),
        Event::Granted(got) => ground.granted(&got, room, (pack, shed)),
        Event::Denied(id) => return Some(ground.denied(id)),
        Event::Kept(tag) => return Some(ground.kept(tag, pack)),
        _ => {}
    }
    None
}

/// What taking the events of the server reads and writes.
type Takes<'w> = (
    ResMut<'w, Ground>,
    ResMut<'w, Pack>,
    ResMut<'w, Shed>,
    Res<'w, Burden>,
);

/// Takes what the server told of the ground this frame, and says in the banner what was refused.
fn receive(
    mut from: MessageReader<FromServer>,
    link: Res<Link>,
    (mut ground, mut pack, mut shed, burden): Takes,
    mut notes: MessageWriter<Note>,
) {
    for FromServer(msg) in from.read() {
        let ServerMsg::Hunt(event) = msg else {
            continue;
        };
        if !grounded(event) {
            continue;
        }
        let into = (&mut *ground, &mut *pack, &mut *shed, burden.room());
        if let Some(line) = hear(event.clone(), link.you, into) {
            notes.write(Note(line.to_string()));
        }
    }
}

/// Keeps the ground alone again once the link is gone, and gives back the drops it never answered.
fn take_back(mut ground: ResMut<Ground>, mut pack: ResMut<Pack>) {
    ground.release(&mut pack);
}

/// Registers the ground of a session. Every system waits for a link, so single player runs none,
/// and for the ground, which a world with no pack has not.
pub(super) fn plugin(app: &mut App) {
    let linked = resource_exists::<Link>;
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
            .chain()
            .run_if(resource_exists::<Ground>),
    );
}
