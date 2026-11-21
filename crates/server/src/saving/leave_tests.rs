//! Tests of saving on a leave: the last place of a player who leaves, or whose connection is
//! lost, reaches the world file at once, even while other players stay.

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use abstractions_net::host::{ConnId, Host, Outbox};
use abstractions_net::room::{Room, RoomConfig};
use abstractions_net::store::{FILE, World};
use abstractions_net::wire::{ClientMsg, Hello, PROTOCOL, PlayerRecord};

use super::Saving;

/// Longest wait for the saver thread to write the world file.
const PATIENCE: Duration = Duration::from_secs(10);

/// Returns a saving room of four seats over an empty folder of its own, named after `tag`.
fn saving(tag: &str) -> (Saving, PathBuf) {
    let name = format!("abstractions-leave-{tag}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a scratch folder");
    let config = RoomConfig {
        name: "Test".to_string(),
        password: None,
        cap: 4,
        version: "0.11.0".to_string(),
    };
    let room = Room::new(config, World::default(), 0.0);
    let saving = Saving::new(room, dir.clone(), 4).expect("a saver thread");
    (saving, dir)
}

/// Joins a player named `name` under the key filled with `key`, on connection `key`.
fn join(saving: &mut Saving, name: &str, key: u8) -> ConnId {
    let conn = ConnId(u64::from(key));
    let addr: SocketAddr = format!("10.0.0.{key}:24650").parse().expect("an address");
    let hello = Hello {
        protocol: PROTOCOL,
        name: name.to_string(),
        key: [key; 32],
        password: None,
    };
    let mut out = Outbox::default();
    saving.connected(conn, addr, 0.0, &mut out);
    saving.message(conn, ClientMsg::Hello(hello), 0.0, &mut out);
    conn
}

/// Sends the record of a player standing `altitude` metres up on connection `conn`.
fn stand(saving: &mut Saving, conn: ConnId, altitude: &str) {
    let place = format!(
        "dir = [0.0, 1.0, 0.0]\naltitude = {altitude}\n\n[body]\nindex = 0\nmoon = 0\n\n\
         [body.star]\ngalaxy = [0, 0, 0]\nsector = [35, 0, -46]\nindex = 0\n"
    );
    let record = PlayerRecord {
        place,
        ..PlayerRecord::default()
    };
    saving.message(conn, ClientMsg::Record(record), 1.0, &mut Outbox::default());
}

/// Returns true once the world file of `dir` holds `text`, waiting for the saver thread.
fn written(dir: &Path, text: &str) -> bool {
    let start = Instant::now();
    while start.elapsed() < PATIENCE {
        let file = fs::read_to_string(dir.join(FILE)).unwrap_or_default();
        if file.contains(text) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

/// A goodbye writes the place of the player who leaves at once, while another player stays:
/// no half minute in which a hard stop would lose it.
#[test]
fn a_goodbye_saves_the_place_while_others_stay() {
    let (mut saving, dir) = saving("bye");
    let ada = join(&mut saving, "Ada", 1);
    join(&mut saving, "Bob", 2);
    stand(&mut saving, ada, "3.25");
    assert!(!dir.join(FILE).exists());
    saving.message(ada, ClientMsg::Bye, 2.0, &mut Outbox::default());
    assert!(written(&dir, "altitude = 3.25"));
    assert!(saving.busy());
    saving.finish().expect("a last save");
    let _ = fs::remove_dir_all(&dir);
}

/// A connection that is lost without a goodbye, as on a timeout, writes the place the player
/// sent last, and the empty server then sleeps: nothing is left to save.
#[test]
fn a_lost_connection_saves_the_place() {
    let (mut saving, dir) = saving("lost");
    let ada = join(&mut saving, "Ada", 1);
    stand(&mut saving, ada, "4.75");
    assert!(saving.busy());
    saving.closed(ada, 2.0, &mut Outbox::default());
    assert!(written(&dir, "altitude = 4.75"));
    assert!(!saving.busy());
    saving.finish().expect("a last save");
    let _ = fs::remove_dir_all(&dir);
}
