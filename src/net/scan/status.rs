//! The status of every saved server, asked on a thread of its own each, since the request
//! blocks, with at most one request in flight per address.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use abstractions_net::io::{CloseReason, addr, status};
use abstractions_net::trust::KnownServers;
use abstractions_net::wire::{PROTOCOL, Status};
use bevy::prelude::*;

use crate::net::{Lobby, NetDir, RowStatus, ServerRow};
use crate::settings::{Persistent, Settings};

/// Longest wait for one server to answer.
const TIMEOUT: Duration = Duration::from_secs(3);

/// What a status request gave: the answer, its round trip and the server's key, or why none.
type Answer = Result<(Status, Duration, [u8; 32]), CloseReason>;

/// One finished request: the address, the answer and the key remembered for that address.
type Done = (String, Answer, Option<[u8; 32]>);

/// The status requests in flight and the answers waiting for the lobby.
#[derive(Resource, Debug, Default)]
pub struct Pings {
    /// Addresses asked and not answered yet.
    flying: HashSet<String>,
    /// Answers the threads left.
    done: Arc<Mutex<Vec<Done>>>,
}

/// Returns the name and the row status of an answer. A server showing another key than the
/// one `known` for its address shows as changed.
pub fn row_status(answer: &Answer, known: Option<[u8; 32]>) -> (String, RowStatus) {
    match answer {
        Ok((_, _, key)) if known.is_some_and(|known| known != *key) => {
            (String::new(), RowStatus::KeyChanged)
        }
        Ok((status, rtt, _)) => (
            status.name.clone(),
            RowStatus::Online {
                players: status.players,
                cap: status.cap,
                ping_ms: u32::try_from(rtt.as_millis()).unwrap_or(u32::MAX),
                version: status.version.clone(),
                compatible: status.protocol == PROTOCOL,
            },
        ),
        Err(_) => (String::new(), RowStatus::Offline),
    }
}

/// Asks `address` for its status on a new thread and leaves the answer in `done`.
fn ask(address: String, known: std::path::PathBuf, done: Arc<Mutex<Vec<Done>>>) {
    let job = move || {
        let answer = status(&address, TIMEOUT);
        let remembered = addr::key(&address).ok().and_then(|key| {
            let servers = KnownServers::load(&known).ok()?;
            servers.get(&key)
        });
        if let Ok(mut done) = done.lock() {
            done.push((address, answer, remembered));
        }
    };
    let _ = thread::Builder::new()
        .name("abstractions-status".to_string())
        .spawn(job);
}

/// Asks every saved server for its status, except those still asked, and shows them as
/// pinging until they answer.
pub fn refresh(
    settings: Res<Persistent<Settings>>,
    dir: Res<NetDir>,
    mut pings: ResMut<Pings>,
    mut lobby: ResMut<Lobby>,
) {
    let mut rows = Vec::with_capacity(settings.servers.len());
    for server in &settings.servers {
        let old = lobby.saved.iter().find(|row| row.address == server.address);
        let mut row = old.cloned().unwrap_or(ServerRow {
            name: String::new(),
            address: server.address.clone(),
            status: RowStatus::Pinging,
            key: None,
        });
        if pings.flying.insert(server.address.clone()) {
            row.status = RowStatus::Pinging;
            ask(server.address.clone(), dir.known(), pings.done.clone());
        }
        rows.push(row);
    }
    lobby.saved = rows;
}

/// Run condition: a status request is in flight.
pub fn flying(pings: Option<Res<Pings>>) -> bool {
    pings.is_some_and(|pings| !pings.flying.is_empty())
}

/// Puts each answer that arrived into the lobby row of its address.
pub fn collect(mut pings: ResMut<Pings>, mut lobby: ResMut<Lobby>) {
    let arrived: Vec<Done> = match pings.done.lock() {
        Ok(mut done) => done.drain(..).collect(),
        Err(_) => return,
    };
    for (address, answer, known) in arrived {
        pings.flying.remove(&address);
        let (name, status) = row_status(&answer, known);
        if let Some(row) = lobby.saved.iter_mut().find(|row| row.address == address) {
            row.name = name;
            row.status = status;
        }
    }
}
