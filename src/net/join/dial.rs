//! Starting a join: the hello, the trust settings, and the link that connects in the background.

use std::io;

use abstractions_net::io::ConnectOpts;
use abstractions_net::wire::{Hello, PROTOCOL};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::open;
use crate::net::link::{Link, Role};
use crate::net::{JoinServer, NetDir, Session, SessionEnded};
use crate::settings::{Persistent, Settings};

/// What a join reads and changes.
#[derive(SystemParam)]
pub struct Dialer<'w, 's> {
    /// Inserts the link and its resources.
    commands: Commands<'w, 's>,
    /// Where the game stands with other players.
    session: ResMut<'w, Session>,
    /// Where the network files are.
    dir: Res<'w, NetDir>,
    /// The player name.
    settings: Option<Res<'w, Persistent<Settings>>>,
    /// Why a join could not start.
    ended: MessageWriter<'w, SessionEnded>,
}

/// Returns what a link needs to join: the hello with the player's name from `settings`, the
/// identity key and `password`, the file of known servers and the key the server must show.
pub fn opts(
    dir: &NetDir,
    settings: Option<&Persistent<Settings>>,
    password: Option<String>,
    expected: Option<[u8; 32]>,
) -> io::Result<ConnectOpts> {
    let name = settings.map(|settings| settings.player_name.trim().to_string());
    let hello = Hello {
        protocol: PROTOCOL,
        name: name.unwrap_or_default(),
        key: dir.identity()?,
        password,
    };
    Ok(ConnectOpts {
        hello,
        known: dir.known(),
        expected,
    })
}

/// Starts joining the last server asked this frame and sets [`Session::Joining`]. A request
/// while joined or hosting is ignored: the session must end first.
pub fn join(mut joins: MessageReader<JoinServer>, mut dialer: Dialer, link: Option<ResMut<Link>>) {
    let Some(request) = joins.read().last().cloned() else {
        return;
    };
    if dialer.session.shared() {
        return;
    }
    if let Some(mut link) = link {
        link.close(false);
    }
    let settings = dialer.settings.as_deref();
    let password = request.password.clone();
    let opts = match opts(&dialer.dir, settings, password, request.expected_key) {
        Ok(opts) => opts,
        Err(error) => {
            let reason = format!("Could not read the player identity: {error}");
            dialer.ended.write(SessionEnded { reason });
            return;
        }
    };
    let role = Role::Join {
        address: request.address.clone(),
        expected: request.expected_key,
    };
    open(
        &mut dialer.commands,
        Link::connect(&request.address, opts, role),
    );
    *dialer.session = Session::Joining {
        address: request.address,
    };
}
