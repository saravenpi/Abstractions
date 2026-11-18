//! Tests of the command line: subcommands, flags, defaults and the environment.

use std::path::PathBuf;

use super::{Command, Options, parse};

/// Returns the arguments of `line`, split on spaces.
fn args(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

/// Returns no variable for any name.
fn none(_: &str) -> Option<String> {
    None
}

/// Returns the options `line` runs with, under an empty environment.
fn options(line: &str) -> Options {
    match parse(&args(line), none) {
        Ok(Command::Run(options)) => options,
        other => panic!("{line}: {other:?}"),
    }
}

/// No argument, or `run` alone, runs on port 24650 with 16 seats and `./data`.
#[test]
fn no_argument_runs_with_the_defaults() {
    let options = options("");
    assert_eq!(options.port, 24650);
    assert_eq!(options.max_players, 16);
    assert_eq!(options.data, PathBuf::from("./data"));
    assert_eq!(
        (options.name, options.password, options.import),
        (None, None, None)
    );
    assert_eq!(parse(&args("run"), none), parse(&args(""), none));
}

/// Every flag is read as `--flag value` and as `--flag=value`.
#[test]
fn every_flag_is_read_in_both_forms() {
    let options = options(
        "run --name Moon --password=hunter2 --port 24700 --max-players=4 --data /srv --import a.toml",
    );
    assert_eq!(options.name.as_deref(), Some("Moon"));
    assert_eq!(options.password.as_deref(), Some("hunter2"));
    assert_eq!(options.port, 24700);
    assert_eq!(options.max_players, 4);
    assert_eq!(options.data, PathBuf::from("/srv"));
    assert_eq!(options.import, Some(PathBuf::from("a.toml")));
    assert_eq!(self::options("--port 1").port, 1);
}

/// A flag wins over its variable, and an empty variable counts as unset.
#[test]
fn a_flag_wins_over_the_environment() {
    let env = |name: &str| match name {
        "ABSTRACTIONS_NAME" => Some("From env".to_string()),
        "ABSTRACTIONS_PORT" => Some("25000".to_string()),
        "ABSTRACTIONS_MAX_PLAYERS" => Some("8".to_string()),
        "ABSTRACTIONS_DATA" => Some("/data".to_string()),
        "ABSTRACTIONS_PASSWORD" => Some(String::new()),
        _ => None,
    };
    let Ok(Command::Run(options)) = parse(&args("--port 26000"), env) else {
        panic!("not a run");
    };
    assert_eq!(options.name.as_deref(), Some("From env"));
    assert_eq!(options.port, 26000);
    assert_eq!(options.max_players, 8);
    assert_eq!(options.data, PathBuf::from("/data"));
    assert_eq!(options.password, None);
}

/// A missing value, a number out of range, an unknown flag or command is refused, and a bad
/// variable of the environment is named in the error.
#[test]
fn bad_flags_are_refused() {
    for line in [
        "--port",
        "--port 0",
        "--port 70000",
        "--max-players 300",
        "--colour red",
        "run extra",
        "fly",
    ] {
        assert!(parse(&args(line), none).is_err(), "{line}");
    }
    let env = |name: &str| (name == "ABSTRACTIONS_PORT").then(|| "high".to_string());
    let error = parse(&args(""), env).unwrap_err();
    assert!(error.starts_with("ABSTRACTIONS_PORT:"), "{error}");
}

/// `version`, `--help` anywhere in `run`, and `ping` with one address or none.
#[test]
fn subcommands_are_read() {
    assert_eq!(parse(&args("version"), none), Ok(Command::Version));
    assert_eq!(parse(&args("--help"), none), Ok(Command::Help));
    assert_eq!(parse(&args("run --port 5 -h"), none), Ok(Command::Help));
    assert_eq!(
        parse(&args("ping example.com:24650"), none),
        Ok(Command::Ping("example.com:24650".to_string()))
    );
    assert_eq!(
        parse(&args("ping"), none),
        Ok(Command::Ping("127.0.0.1:24650".to_string()))
    );
    let port = |name: &str| (name == "ABSTRACTIONS_PORT").then(|| "24700".to_string());
    assert_eq!(
        parse(&args("ping"), port),
        Ok(Command::Ping("127.0.0.1:24700".to_string()))
    );
    assert!(parse(&args("ping a b"), none).is_err());
}
