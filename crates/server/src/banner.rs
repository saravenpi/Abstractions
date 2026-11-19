//! The banner the server prints once it listens: what it runs and how to join it.

use std::fmt::Write;
use std::net::IpAddr;
use std::path::Path;

/// Width of the label column of the banner.
const LABEL: usize = 10;

/// What the banner shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Banner<'a> {
    /// Version of the server program.
    pub version: &'a str,
    /// Name of the server in the list.
    pub name: &'a str,
    /// Seed of the universe.
    pub seed: u64,
    /// Path of the world file.
    pub file: &'a Path,
    /// UDP port the server listens on.
    pub port: u16,
    /// Address of this machine on its network, when it could be found.
    pub lan: Option<IpAddr>,
    /// Short key of the server's certificate.
    pub key: &'a str,
    /// Most players at once.
    pub players: u16,
    /// True when players need a password. The password itself is never shown.
    pub password: bool,
}

/// Returns the banner as lines ending in a newline.
pub fn banner(info: &Banner<'_>) -> String {
    let port = info.port;
    let join = match info.lan {
        Some(ip) => format!("{ip}:{port}"),
        None => format!("this machine's address, port {port}"),
    };
    let mut text = format!("Abstractions server v{}\n", info.version);
    let world = format!(
        "{} (seed {})   {}",
        info.name,
        info.seed,
        info.file.display()
    );
    row(&mut text, "World", &world);
    let listening = format!("UDP {port} on all addresses (0.0.0.0)");
    row(&mut text, "Listening", &listening);
    let players = format!("up to {} at once", info.players);
    row(&mut text, "Players", &players);
    row(&mut text, "Join at", &join);
    row(
        &mut text,
        "Key",
        &format!("{}   players see this the first time they join", info.key),
    );
    if info.password {
        row(&mut text, "Password", "on");
    }
    let internet = format!("forward UDP port {port} to this machine, or point a domain at it");
    row(&mut text, "Internet", &internet);
    text
}

/// Adds one line of `label` and `value` to `text`.
fn row(text: &mut String, label: &str, value: &str) {
    let _ = writeln!(text, "{label:<LABEL$}{value}");
}
