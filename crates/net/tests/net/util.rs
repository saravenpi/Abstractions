//! Helpers of the tests: folders, servers, players and waiting for events.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use abstractions_net::io::{Client, ClientEvent, ConnectOpts, ServeConfig, ServerHandle, serve};
use abstractions_net::wire::{Hello, PROTOCOL};

use super::echo::Echo;

/// Returns a fresh folder under the system's temporary folder.
pub fn scratch(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "abstractions-io-{name}-{}-{stamp}",
        std::process::id()
    ))
}

/// Starts a test server on `bind` with its identity in `dir`.
pub fn start(dir: &Path, bind: SocketAddr) -> ServerHandle<Echo> {
    let config = ServeConfig {
        bind,
        dir: dir.to_path_buf(),
    };
    serve(config, Echo::default()).unwrap()
}

/// Returns the options of a player named `name` who keeps its known servers in `known`.
pub fn opts(known: &Path, name: &str) -> ConnectOpts {
    ConnectOpts {
        hello: Hello {
            protocol: PROTOCOL,
            name: name.to_string(),
            key: [1; 32],
            password: None,
        },
        known: known.to_path_buf(),
        expected: None,
    }
}

/// Polls `client` until `pick` takes an event, skipping the others, for at most `within`.
pub fn until<T>(
    client: &mut Client,
    within: Duration,
    mut pick: impl FnMut(ClientEvent) -> Option<T>,
) -> T {
    let end = Instant::now() + within;
    while Instant::now() < end {
        match client.poll() {
            Some(event) => {
                if let Some(found) = pick(event) {
                    return found;
                }
            }
            None => std::thread::sleep(Duration::from_millis(5)),
        }
    }
    panic!("no such event within {within:?}");
}

/// Waits for the connection of `client` and returns whether the server was new.
pub fn connected(client: &mut Client) -> bool {
    until(client, Duration::from_secs(10), |event| match event {
        ClientEvent::Connected { first_time, .. } => Some(first_time),
        ClientEvent::Closed(reason) => panic!("closed: {reason:?}"),
        _ => None,
    })
}

/// Waits for the end of the connection of `client` and returns why it ended.
pub fn closed(client: &mut Client, within: Duration) -> abstractions_net::io::CloseReason {
    until(client, within, |event| match event {
        ClientEvent::Closed(reason) => Some(reason),
        _ => None,
    })
}
