//! Tests of the player cap the owner sets, by flag or by environment.

use super::{Command, parse};

/// Returns the cap `line` runs with, under `env`, or the error.
fn cap_in(line: &str, env: impl Fn(&str) -> Option<String>) -> Result<u16, String> {
    let args: Vec<String> = line.split_whitespace().map(str::to_string).collect();
    match parse(&args, env)? {
        Command::Run(options) => Ok(options.max_players),
        other => Err(format!("{other:?}")),
    }
}

/// Returns the cap `line` runs with under an empty environment.
fn cap(line: &str) -> u16 {
    cap_in(line, |_| None).expect("runs")
}

/// A server takes as many players as its owner asks, up to a thousand, with 16 by default.
#[test]
fn the_player_cap_follows_the_owner() {
    assert_eq!(cap("--max-players 300"), 300);
    assert_eq!(cap("--max-players 1000"), 1000);
    assert_eq!(cap(""), 16);
    let env = |value: &'static str| {
        move |name: &str| (name == "ABSTRACTIONS_MAX_PLAYERS").then(|| value.to_string())
    };
    assert_eq!(cap_in("", env("250")), Ok(250));
    assert!(cap_in("", env("5000")).is_err());
    assert!(cap_in("--max-players 1001", |_| None).is_err());
}
