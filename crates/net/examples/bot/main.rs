//! A load-test and test-partner bot: up to 15 players who walk in circles.
//!
//! `cargo run -p abstractions-net --example bot -- <address> [--count N] [--seconds S]
//! [--password P]` joins `N` players named `Bot 1`, `Bot 2`.. Each sends a robot record at
//! 30 Hz, walking a 20 m circle on the home body, and a ping every second. Every second the bot
//! prints how many are in, how many records came back and the average round trip.
//!
//! A server takes at most 8 connections per IP address. Give several addresses of the same
//! server, separated by commas, to spread more bots: `127.0.0.1,192.168.1.20`.

mod args;
mod player;

use std::thread;
use std::time::{Duration, Instant};

use args::Args;
use player::{Player, Tally};

/// Sends per second of each bot.
const RATE: u32 = 30;

/// Runs the bots until the time is up, then says how many were in at the end.
fn main() {
    let args = match args::parse(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}\n{}", args::USAGE);
            std::process::exit(2);
        }
    };
    let known = std::env::temp_dir().join(format!("abstractions-bot-{}.toml", std::process::id()));
    let mut players: Vec<Player> = (0..args.count)
        .map(|index| Player::new(index, &args, &known))
        .collect();
    let connected = drive(&mut players, &args);
    players.into_iter().for_each(Player::close);
    let _ = std::fs::remove_file(&known);
    println!("Done: {connected}/{} connected at the end", args.count);
    if connected < args.count {
        std::process::exit(1);
    }
}

/// Steps every bot at [`RATE`] and prints a line each second, for the time asked or forever.
/// Returns how many were in at the last second.
fn drive(players: &mut [Player], args: &Args) -> usize {
    let start = Instant::now();
    let mut tally = Tally::default();
    let mut connected = 0;
    for step in 0u64.. {
        let now = start.elapsed();
        if args.seconds.is_some_and(|seconds| now.as_secs() >= seconds) {
            break;
        }
        let second = step % u64::from(RATE) == 0;
        for player in players.iter_mut() {
            player.step(now, second, &mut tally);
        }
        if second && step > 0 {
            connected = players.iter().filter(|player| player.joined()).count();
            println!(
                "{:>4} s  {}",
                now.as_secs(),
                tally.line(connected, players.len())
            );
            tally = Tally::default();
        }
        let next = start + Duration::from_secs_f64((step + 1) as f64 / f64::from(RATE));
        thread::sleep(next.saturating_duration_since(Instant::now()));
    }
    connected
}
