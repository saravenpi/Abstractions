//! Opening the world being played to the local network.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::ops::RangeInclusive;
use std::path::Path;
use std::time::Instant;

use abstractions_net::io::{ServeConfig, ServerHandle, serve};
use abstractions_net::lan::{Announcer, Beacon};
use abstractions_net::room::{Room, RoomConfig};
use abstractions_net::store::World;
use abstractions_net::trust::short_key;
use abstractions_net::wire::{GAME_PORT, MAX_PLAYERS_DEFAULT, MAX_PLAYERS_LIMIT, PROTOCOL};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::Hosted;
use crate::computer::Atlas;
use crate::map::explored::Explored;
use crate::net::join::{open, opts};
use crate::net::link::{Link, Role};
use crate::net::{NetDir, OpenToLan, Session, SessionNotice};
use crate::saves::ActiveWorld;
use crate::settings::{Persistent, Settings};
use crate::sim::Changes;
use crate::space::Clock;

/// The ports a host tries, in order: the game port and the ten after it.
pub const PORTS: RangeInclusive<u16> = GAME_PORT..=GAME_PORT + 10;

/// Returns the address of this machine on its local network, or the loopback address when it
/// has none. Connecting a UDP socket sends nothing; it only picks the route.
pub fn lan_ip() -> IpAddr {
    let probe = UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).and_then(|socket| {
        socket.connect(SocketAddr::from((Ipv4Addr::new(192, 168, 0, 1), 9)))?;
        socket.local_addr()
    });
    probe.map_or(IpAddr::V4(Ipv4Addr::LOCALHOST), |addr| addr.ip())
}

/// Starts a server for `world` on the first free port of [`PORTS`], keeping its certificate
/// in `dir`.
fn bind(config: &RoomConfig, world: &World, dir: &Path) -> io::Result<ServerHandle<Room>> {
    let mut last = io::Error::from(io::ErrorKind::AddrInUse);
    for port in PORTS {
        let room = Room::new(config.clone(), world.clone(), 0.0);
        let at = ServeConfig {
            bind: SocketAddr::from((Ipv4Addr::UNSPECIFIED, port)),
            dir: dir.to_path_buf(),
        };
        match serve(at, room) {
            Ok(server) => return Ok(server),
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => last = error,
            Err(error) => return Err(error),
        }
    }
    Err(last)
}

/// Returns the most players a world opened to LAN takes: the setting, from 1 to
/// [`MAX_PLAYERS_LIMIT`], or the default without settings.
fn host_cap(settings: Option<&Settings>) -> u16 {
    let asked = settings.map_or(MAX_PLAYERS_DEFAULT, |settings| settings.host_max_players);
    asked.clamp(1, MAX_PLAYERS_LIMIT)
}

/// Returns the beacon of a world served on `port` with the certificate `key`, with the host
/// as its one player.
fn beacon_of(config: RoomConfig, port: u16, key: [u8; 32]) -> Beacon {
    Beacon {
        protocol: PROTOCOL,
        version: config.version,
        name: config.name,
        players: 1,
        cap: config.cap,
        port,
        key,
    }
}

/// What opening to LAN reads and changes.
#[derive(SystemParam)]
pub struct Opening<'w, 's> {
    /// Inserts the server and the link.
    commands: Commands<'w, 's>,
    /// Where the game stands with other players.
    session: ResMut<'w, Session>,
    /// Where the network files are.
    dir: Res<'w, NetDir>,
    /// The player name.
    settings: Option<Res<'w, Persistent<Settings>>>,
    /// The world being played.
    active: Option<Res<'w, ActiveWorld>>,
    /// The universe time.
    clock: Res<'w, Clock>,
    /// The log of changes.
    changes: Option<Res<'w, Changes>>,
    /// What the player discovered.
    atlas: Option<Res<'w, Atlas>>,
    /// The ground the player uncovered.
    explored: Option<Res<'w, Explored>>,
}

impl Opening<'_, '_> {
    /// Returns the server's world of the world being played: its name, seed, universe time,
    /// log, atlas and explored ground, with no player records. Nothing without a world.
    fn world(&self) -> Option<World> {
        let save = &self.active.as_deref()?.0.save;
        let atlas = self
            .atlas
            .as_deref()
            .and_then(|atlas| toml::to_string(atlas).ok());
        let explored = self
            .explored
            .as_deref()
            .into_iter()
            .flat_map(Explored::bodies);
        Some(World {
            name: save.name.clone(),
            seed: save.seed,
            clock: self.clock.seconds,
            changes: self.changes.as_deref().cloned().unwrap_or_default(),
            atlas: atlas.unwrap_or_default(),
            explored: explored
                .map(|(body, leaves)| (body, leaves.runs()))
                .collect(),
            ..World::default()
        })
    }

    /// Serves `world`, announces it, joins it as a normal player and sets
    /// [`Session::Hosting`].
    fn host(&mut self, world: World) -> io::Result<()> {
        let config = RoomConfig {
            name: world.name.clone(),
            password: None,
            cap: host_cap(self.settings.as_deref().map(Persistent::get)),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let mut joined = opts(&self.dir, self.settings.as_deref(), None, None)?;
        let server = bind(&config, &world, &self.dir.host())?;
        let (port, key) = (server.local_addr().port(), server.fingerprint());
        joined.expected = Some(key);
        let beacon = beacon_of(config, port, key);
        let announcer = Announcer::start(beacon.clone())
            .inspect_err(|error| warn!("cannot announce the world on the network: {error}"))
            .ok();
        let link = Link::connect(&format!("127.0.0.1:{port}"), joined, Role::Host);
        open(&mut self.commands, link);
        self.commands.insert_resource(Hosted {
            server: Some(server),
            announcer,
            beacon,
            started: Instant::now(),
        });
        *self.session = Session::Hosting {
            lan_address: format!("{}:{port}", lan_ip()),
            key: short_key(&key),
        };
        Ok(())
    }
}

/// Opens the single-player world being played to the local network. The world goes on as it
/// was: nothing restarts.
pub fn open_lan(
    mut opens: MessageReader<OpenToLan>,
    mut opening: Opening,
    mut notices: MessageWriter<SessionNotice>,
) {
    if opens.read().last().is_none() || *opening.session != Session::Offline {
        return;
    }
    let Some(world) = opening.world() else {
        return;
    };
    if let Err(error) = opening.host(world) {
        warn!("cannot open the world to the network: {error}");
        let line = format!("Could not open to LAN: {error}");
        notices.write(SessionNotice { line });
    }
}

#[cfg(test)]
#[path = "open_tests.rs"]
mod tests;
