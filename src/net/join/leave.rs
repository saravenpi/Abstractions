//! What the server keeps of the player: the place, the ship and the pack, sent every 10 seconds,
//! on debounced pack changes, on leaving and on quitting the game, and the goodbye itself.

use std::time::Duration;

use abstractions_net::wire::ClientMsg;
use bevy::prelude::*;

use super::{RECORD_EVERY, record_of, shut};
use crate::craft::Pack;
use crate::net::link::{Link, Role};
use crate::net::{LeaveSession, Players, Session};
use crate::saves::Snapshot;
use crate::ship::Fleet;

/// Debounce delay before sending a changed pack to the server.
const PACK_DEBOUNCE: Duration = Duration::from_secs(2);

/// Periodic and debounced pack-change timers for [`keep_record`].
#[derive(Default)]
pub struct RecordTimers {
    /// Repeating timer for periodic player record sync.
    every: Option<Timer>,
    /// One-shot debounce timer armed when the pack changes.
    pack: Option<Timer>,
}

/// Captures the world as a save would, and sends the player's place, ship and pack to the server.
/// Only a player the server welcomed to its world has a record to keep, and a player flying
/// someone else's ship leaves their own saved ship untouched.
fn send_record(
    link: &mut Link,
    snapshot: &mut Snapshot,
    fleet: Option<&Fleet>,
    pack: Option<&Pack>,
) {
    let joined = matches!(link.role, Role::Join { .. }) && link.you.is_some();
    let Some(active) = snapshot.capture().filter(|_| joined) else {
        return;
    };
    let mut record = record_of(&active.0.save);
    if let Some(pack) = pack {
        record.pack = toml::to_string(pack).unwrap_or_default();
    }
    if fleet.is_some_and(|f| f.active != link.you) {
        record.ship.clear();
    }
    link.send(ClientMsg::Record(record));
}

/// Sends the player's record every [`RECORD_EVERY`] while joined, or after [`PACK_DEBOUNCE`]
/// following a pack change.
pub fn keep_record(
    time: Res<Time<Real>>,
    mut timers: Local<RecordTimers>,
    (mut link, fleet, pack): (ResMut<Link>, Option<Res<Fleet>>, Option<Res<Pack>>),
    mut snapshot: Snapshot,
) {
    let delta = time.delta();
    if pack.as_ref().is_some_and(Res::is_changed) {
        timers.pack = Some(Timer::new(PACK_DEBOUNCE, TimerMode::Once));
    }
    let every = timers
        .every
        .get_or_insert_with(|| Timer::new(RECORD_EVERY, TimerMode::Repeating));
    let due_periodic = every.tick(delta).just_finished();
    let due_pack = timers
        .pack
        .as_mut()
        .is_some_and(|t| t.tick(delta).just_finished());
    if due_periodic || due_pack {
        timers.pack = None;
        send_record(&mut link, &mut snapshot, fleet.as_deref(), pack.as_deref());
    }
}

/// Leaves the server: the record, the goodbye, and back to single player. The interface
/// itself goes to the title screen or back to the list. A host stops with `CloseLan` instead.
pub fn leave(
    mut leaves: MessageReader<LeaveSession>,
    (mut link, fleet, pack): (ResMut<Link>, Option<Res<Fleet>>, Option<Res<Pack>>),
    mut snapshot: Snapshot,
    mut players: ResMut<Players>,
    mut commands: Commands,
) {
    if leaves.read().last().is_none() || link.role == Role::Host {
        return;
    }
    send_record(&mut link, &mut snapshot, fleet.as_deref(), pack.as_deref());
    link.close(false);
    shut(&mut commands);
    players.0.clear();
    commands.insert_resource(Session::Offline);
}

/// Sends the record and the goodbye when the game quits while joined, and waits for them to
/// leave. A join still connecting is dropped without waiting. A host is stopped by the
/// hosting systems.
pub fn quit(
    mut exits: MessageReader<AppExit>,
    (mut link, fleet, pack): (ResMut<Link>, Option<Res<Fleet>>, Option<Res<Pack>>),
    mut snapshot: Snapshot,
) {
    if exits.read().next().is_none() || link.role == Role::Host {
        return;
    }
    send_record(&mut link, &mut snapshot, fleet.as_deref(), pack.as_deref());
    let welcomed = link.you.is_some();
    link.close(welcomed);
}
