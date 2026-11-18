//! The self-hosted server of Abstractions: one world, up to sixteen players, no setup.
//!
//! `abstractions-server` in an empty folder makes `./data/world.toml`, a certificate, and
//! listens on UDP 24650. `ping` asks any server what it is, `version` prints the version.

mod banner;
mod cli;
mod lan;
mod ping;
mod run;
mod saving;
mod signal;

#[cfg(test)]
mod banner_tests;
#[cfg(test)]
mod ping_tests;

use std::process::ExitCode;

use cli::{Command, USAGE};

/// Version of the server program, such as `0.9.0`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reads the command line and the environment, then runs what they ask.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match cli::parse(&args, |name| std::env::var(name).ok()) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("{error}\nTry `abstractions-server --help`.");
            return ExitCode::from(2);
        }
    };
    match command {
        Command::Help => print!("{USAGE}"),
        Command::Version => println!("abstractions-server {VERSION}"),
        Command::Ping(address) => return ping::ping(&address),
        Command::Run(options) => {
            if let Err(error) = run::run(options) {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}
