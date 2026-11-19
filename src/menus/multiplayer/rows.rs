//! The rows of the multiplayer menu: what each server shows, built from the settings and the
//! lobby the network fills.

use bevy::prelude::*;

use super::{Listed, Section, Servers};
use crate::net::{Lobby, RowStatus};
use crate::settings::{Persistent, SavedServer, Settings};
use crate::ui::Tone;

/// A row button, by its index in the list.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Row(pub usize);

/// Which text of a row a cell is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    /// The name of the server, or its address when it has none.
    Name,
    /// The address, muted under the name.
    Address,
    /// Players and ping, or why the server cannot be joined.
    Status,
}

/// A text of the row at this index.
#[derive(Component, Debug, Clone, Copy)]
pub(super) struct Cell(pub usize, pub Part);

/// Returns the rows of the menu: the saved servers with the status the lobby has for their
/// address, then the games found on the network.
pub(super) fn listed(saved: &[SavedServer], lobby: &Lobby) -> Vec<Listed> {
    let status = |address: &str| {
        let found = lobby.saved.iter().find(|row| row.address == address);
        found.map_or(RowStatus::Pinging, |row| row.status.clone())
    };
    let mine = saved.iter().enumerate().map(|(index, server)| Listed {
        section: Section::Saved(index),
        name: server.name.clone(),
        address: server.address.clone(),
        status: status(&server.address),
        key: None,
    });
    let found = lobby.lan.iter().map(|row| Listed {
        section: Section::Lan,
        name: row.name.clone(),
        address: row.address.clone(),
        status: row.status.clone(),
        key: row.key,
    });
    mine.chain(found).collect()
}

/// Returns the right-hand text of a row and its tone: "3/16  42 ms", "Needs v0.9.0",
/// "Offline", "Key changed", or "Pinging" while the answer is on its way.
pub(super) fn status_text(status: &RowStatus) -> (String, Tone) {
    match status {
        RowStatus::Pinging => ("Pinging".to_string(), Tone::Muted),
        RowStatus::Online {
            compatible: false,
            version,
            ..
        } => (format!("Needs v{version}"), Tone::Muted),
        RowStatus::Online {
            players,
            cap,
            ping_ms,
            ..
        } => (format!("{players}/{cap}  {ping_ms} ms"), Tone::Plain),
        RowStatus::Offline => ("Offline".to_string(), Tone::Muted),
        RowStatus::KeyChanged => ("Key changed".to_string(), Tone::Danger),
    }
}

/// Returns the text and the tone of one cell of `server`.
pub(super) fn cell_text(server: &Listed, part: Part) -> (String, Tone) {
    match part {
        Part::Name if server.name.trim().is_empty() => (server.address.clone(), Tone::Plain),
        Part::Name => (server.name.clone(), Tone::Plain),
        Part::Address => (server.address.clone(), Tone::Muted),
        Part::Status => status_text(&server.status),
    }
}

/// Builds the rows again when the saved servers or the lobby changed.
pub(super) fn refresh(
    settings: Res<Persistent<Settings>>,
    lobby: Res<Lobby>,
    mut servers: ResMut<Servers>,
) {
    if !(settings.is_changed() || lobby.is_changed()) {
        return;
    }
    let list = listed(&settings.servers, &lobby);
    if list != servers.list {
        servers.fill(list);
    }
}

/// Writes the texts of every row. Does nothing while neither the rows nor the cells changed.
pub(super) fn show(
    servers: Res<Servers>,
    fresh: Query<(), Added<Cell>>,
    mut cells: Query<(&Cell, &mut Text, &mut Tone)>,
) {
    if !(servers.is_changed() || !fresh.is_empty()) {
        return;
    }
    for (Cell(index, part), mut text, mut tone) in &mut cells {
        let Some(server) = servers.list.get(*index) else {
            continue;
        };
        let (shown, wanted) = cell_text(server, *part);
        text.set_if_neq(Text(shown));
        tone.set_if_neq(wanted);
    }
}
