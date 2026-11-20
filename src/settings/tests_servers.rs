//! Tests that the player name, the saved servers and the host cap load from files of every
//! age.

use super::{SavedServer, Settings, default_servers, load_from};

/// A settings file written before multiplayer: no player name and no server list.
const BEFORE: &str = "[graphics]\nfov = 90\n\n[audio]\nmusic = 0.25\n";

/// A file without the multiplayer keys keeps its values and gets an empty name and the public
/// server, on disk as in memory.
#[test]
fn files_without_the_multiplayer_keys_load() {
    let parsed: Settings = toml::from_str(BEFORE).unwrap();
    assert_eq!((parsed.graphics.fov, parsed.audio.music), (90, 0.25));
    assert_eq!(parsed.player_name, "");
    assert_eq!(parsed.servers, default_servers());
    assert_eq!(parsed.servers[0].address, "space.facile.studio");
    assert_eq!(parsed.host_max_players, 16);
    let path = std::env::temp_dir()
        .join(format!("abstractions-test-{}-servers", std::process::id()))
        .join("settings.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, BEFORE).unwrap();
    assert_eq!(*load_from(path.clone()).get(), parsed);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

/// A removed public server stays removed, and a saved list and name survive a round trip.
#[test]
fn saved_servers_and_the_name_round_trip() {
    let emptied: Settings = toml::from_str("servers = []\n").unwrap();
    assert!(emptied.servers.is_empty());
    let settings = Settings {
        player_name: "Ada".to_string(),
        host_max_players: 120,
        servers: vec![SavedServer {
            name: String::new(),
            address: "10.0.0.2:24650".to_string(),
        }],
        ..Settings::default()
    };
    let text = toml::to_string(&settings).unwrap();
    assert_eq!(toml::from_str::<Settings>(&text).unwrap(), settings);
}
