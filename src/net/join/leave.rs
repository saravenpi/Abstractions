//! What the server keeps of the player: the place and the ship, sent every 30 seconds, on
//! leaving and on quitting the game, and the goodbye itself.

use abstractions_net::wire::ClientMsg;
use bevy::prelude::*;

use super::{RECORD_EVERY, record_of, shut};
use crate::net::link::{Link, Role};
use crate::net::{LeaveSession, Players, Session};
use crate::saves::Snapshot;

/// Captures the world as a save would, and sends the player's place and ship to the server.
/// Only a player the server welcomed to its world has a record to keep.
fn send_record(link: &mut Link, snapshot: &mut Snapshot) {
    let joined = matches!(link.role, Role::Join { .. }) && link.you.is_some();
    let Some(active) = snapshot.capture().filter(|_| joined) else {
        return;
    };
    let record = record_of(&active.0.save);
    link.send(ClientMsg::Record(record));
}

/// Sends the player's record every [`RECORD_EVERY`] while joined.
pub fn keep_record(
    time: Res<Time<Real>>,
    mut every: Local<Option<Timer>>,
    mut link: ResMut<Link>,
    mut snapshot: Snapshot,
) {
    let timer = every.get_or_insert_with(|| Timer::new(RECORD_EVERY, TimerMode::Repeating));
    if timer.tick(time.delta()).just_finished() {
        send_record(&mut link, &mut snapshot);
    }
}

/// Leaves the server: the record, the goodbye, and back to single player. The interface
/// itself goes to the title screen or back to the list. A host stops with `CloseLan` instead.
pub fn leave(
    mut leaves: MessageReader<LeaveSession>,
    mut link: ResMut<Link>,
    mut snapshot: Snapshot,
    mut players: ResMut<Players>,
    mut commands: Commands,
) {
    if leaves.read().last().is_none() || link.role == Role::Host {
        return;
    }
    send_record(&mut link, &mut snapshot);
    link.close(false);
    shut(&mut commands);
    players.0.clear();
    commands.insert_resource(Session::Offline);
}

/// Sends the record and the goodbye when the game quits while joined, and waits for them to
/// leave. A join still connecting is dropped without waiting. A host is stopped by the
/// hosting systems.
pub fn quit(mut exits: MessageReader<AppExit>, mut link: ResMut<Link>, mut snapshot: Snapshot) {
    if exits.read().next().is_none() || link.role == Role::Host {
        return;
    }
    send_record(&mut link, &mut snapshot);
    let welcomed = link.you.is_some();
    link.close(welcomed);
}
