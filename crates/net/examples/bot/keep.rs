//! What a bot takes from the environment: its identity key and the record it keeps on the server.
//!
//! `BOT_KEY=<n>` makes the key of every bot a function of `<n>`, so a second run is the same
//! player. `BOT_KEY_FILE=<path>` takes the 32 bytes of a game's `identity.key` for bot 1, so the
//! bot is that player. `BOT_PLACE_FILE` and `BOT_SHIP_FILE` name text files sent to the server as
//! the record of the player once welcomed.

use std::fs;

use abstractions_net::wire::PlayerRecord;

/// Returns the identity key of bot number `index`.
pub fn key(index: usize) -> [u8; 32] {
    if index == 0
        && let Some(bytes) = std::env::var("BOT_KEY_FILE")
            .ok()
            .and_then(|path| fs::read(path).ok())
        && let Ok(key) = <[u8; 32]>::try_from(bytes.as_slice())
    {
        return key;
    }
    let seed = std::env::var("BOT_KEY")
        .ok()
        .and_then(|text| text.parse::<u32>().ok())
        .unwrap_or_else(std::process::id);
    let mut key = [0xb0; 32];
    key[..4].copy_from_slice(&(index as u32).to_le_bytes());
    key[4..8].copy_from_slice(&seed.to_le_bytes());
    key
}

/// Returns the record the bots send once welcomed, when a file names one.
pub fn record() -> Option<PlayerRecord> {
    let place = std::env::var("BOT_PLACE_FILE").ok();
    let ship = std::env::var("BOT_SHIP_FILE").ok();
    if place.is_none() && ship.is_none() {
        return None;
    }
    let read = |path: Option<String>| {
        path.and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default()
    };
    Some(PlayerRecord {
        place: read(place),
        ship: read(ship),
        marks: String::new(),
    })
}

/// Returns one line that says what the server kept of the player, for the log.
pub fn said(kept: Option<&PlayerRecord>) -> String {
    let line = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    match kept {
        None => "welcome: no record kept".to_string(),
        Some(record) => format!(
            "welcome: record kept place [{}] ship [{}]",
            line(&record.place),
            line(&record.ship)
        ),
    }
}
