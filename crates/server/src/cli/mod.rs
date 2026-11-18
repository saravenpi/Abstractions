//! The command line: subcommands, flags, and the environment variables the flags override.

mod usage;

#[cfg(test)]
mod parse_tests;

pub use usage::USAGE;

use std::path::PathBuf;
use std::str::FromStr;

use abstractions_net::wire::{GAME_PORT, MAX_PLAYERS_DEFAULT};

/// Data folder when neither `--data` nor `ABSTRACTIONS_DATA` names one.
const DATA: &str = "./data";

/// Each flag a variable of the environment can set, with that variable.
const ENV: [(&str, &str); 5] = [
    ("--name", "ABSTRACTIONS_NAME"),
    ("--password", "ABSTRACTIONS_PASSWORD"),
    ("--port", "ABSTRACTIONS_PORT"),
    ("--max-players", "ABSTRACTIONS_MAX_PLAYERS"),
    ("--data", "ABSTRACTIONS_DATA"),
];

/// What the owner asked the program to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run the server.
    Run(Options),
    /// Ask the server at this address what it is.
    Ping(String),
    /// Print the version.
    Version,
    /// Print the short help.
    Help,
}

/// How to run the server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Name in the server list, or nothing to use the world's name.
    pub name: Option<String>,
    /// Password a player must give, or nothing for an open server.
    pub password: Option<String>,
    /// UDP port to listen on.
    pub port: u16,
    /// Most players at once.
    pub max_players: u8,
    /// Folder of the world file and the certificate.
    pub data: PathBuf,
    /// Single-player world file to start the server's world from.
    pub import: Option<PathBuf>,
}

/// Reads the arguments after the program name. `env` returns a variable of the environment;
/// an empty one counts as unset. A flag wins over its variable.
pub fn parse(args: &[String], env: impl Fn(&str) -> Option<String>) -> Result<Command, String> {
    match args.first().map(String::as_str) {
        None => run(args, &env),
        Some("run") => run(&args[1..], &env),
        Some("ping") => ping(&args[1..]),
        Some("version" | "--version" | "-V") => Ok(Command::Version),
        Some("help" | "--help" | "-h") => Ok(Command::Help),
        Some(flag) if flag.starts_with("--") => run(args, &env),
        Some(other) => Err(format!("Unknown command `{other}`")),
    }
}

/// Reads the flags of `run` over what the environment set.
fn run(args: &[String], env: &impl Fn(&str) -> Option<String>) -> Result<Command, String> {
    let mut options = defaults(env)?;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            return Ok(Command::Help);
        }
        let (flag, value) = match arg.split_once('=') {
            Some((flag, value)) => (flag, Some(value.to_string())),
            None => (arg.as_str(), None),
        };
        let value = value.or_else(|| args.next().cloned());
        set(&mut options, flag, value)?;
    }
    Ok(Command::Run(options))
}

/// Returns the built-in options with every variable of the environment that is set applied.
fn defaults(env: &impl Fn(&str) -> Option<String>) -> Result<Options, String> {
    let mut options = Options {
        name: None,
        password: None,
        port: GAME_PORT,
        max_players: MAX_PLAYERS_DEFAULT,
        data: PathBuf::from(DATA),
        import: None,
    };
    for (flag, var) in ENV {
        if let Some(value) = env(var).filter(|value| !value.is_empty()) {
            set(&mut options, flag, Some(value)).map_err(|error| format!("{var}: {error}"))?;
        }
    }
    Ok(options)
}

/// Sets the option of `flag` to `value`.
fn set(options: &mut Options, flag: &str, value: Option<String>) -> Result<(), String> {
    let value = move || value.ok_or_else(|| format!("{flag} needs a value"));
    match flag {
        "--name" => options.name = Some(value()?),
        "--password" => options.password = Some(value()?),
        "--port" => options.port = number(flag, &value()?)?,
        "--max-players" => options.max_players = number(flag, &value()?)?,
        "--data" => options.data = PathBuf::from(value()?),
        "--import" => options.import = Some(PathBuf::from(value()?)),
        _ => return Err(format!("Unknown option `{flag}`")),
    }
    Ok(())
}

/// Returns `text` as a number above zero that fits `T`.
fn number<T: FromStr + PartialOrd + Default>(flag: &str, text: &str) -> Result<T, String> {
    text.parse()
        .ok()
        .filter(|number| *number > T::default())
        .ok_or_else(|| format!("{flag} takes a whole number from 1 up, not `{text}`"))
}

/// Reads the one address of `ping`.
fn ping(args: &[String]) -> Result<Command, String> {
    match args {
        [address] => Ok(Command::Ping(address.clone())),
        _ => Err(
            "ping takes one address, such as `ping 127.0.0.1` or `ping example.com:24650`"
                .to_string(),
        ),
    }
}
