//! Server addresses as a player types them: `host`, `host:port`, `1.2.3.4:port`, `[::1]:port`.

use std::fmt;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::wire::GAME_PORT;

/// Why a typed address names no server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddrError {
    /// Nothing was typed.
    Empty,
    /// The text is not a host or a host and a port.
    Malformed,
    /// The port is not a number from 1 to 65535.
    BadPort,
    /// The name did not resolve to any address.
    NotFound,
}

impl fmt::Display for AddrError {
    /// Writes the error in plain words, ready for the screen.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "No address was given",
            Self::Malformed => "This is not an address",
            Self::BadPort => "The port must be a number from 1 to 65535",
            Self::NotFound => "No server has this name",
        })
    }
}

impl std::error::Error for AddrError {}

/// Turns typed text into the address of a server, on [`GAME_PORT`] unless a port is given.
///
/// A name is looked up through the system resolver, which blocks: call this off the game's
/// main thread, as [`super::Client`] does. An IPv4 address is preferred when a name has both.
pub fn resolve(text: &str) -> Result<SocketAddr, AddrError> {
    let (host, port) = split(text)?;
    let found: Vec<SocketAddr> = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|_| AddrError::NotFound)?
        .collect();
    let first = found.iter().find(|addr| addr.is_ipv4()).or(found.first());
    first.copied().ok_or(AddrError::NotFound)
}

/// Returns the address in one written form, `host:port` or `[v6]:port`, with the host in
/// lowercase. Known servers are remembered under this form.
pub fn key(text: &str) -> Result<String, AddrError> {
    let (host, port) = split(text)?;
    Ok(match host.contains(':') {
        true => format!("[{host}]:{port}"),
        false => format!("{host}:{port}"),
    })
}

/// Splits typed text into a lowercase host, without brackets, and a port.
fn split(text: &str) -> Result<(String, u16), AddrError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(AddrError::Empty);
    }
    if let Ok(addr) = text.parse::<SocketAddr>() {
        return port(addr.port()).map(|port| (addr.ip().to_string(), port));
    }
    let bare = text
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'));
    if let Ok(ip) = bare.unwrap_or(text).parse::<IpAddr>() {
        return Ok((ip.to_string(), GAME_PORT));
    }
    let (host, number) = match text.rsplit_once(':') {
        Some((host, number)) => (host, number.parse().map_err(|_| AddrError::BadPort)?),
        None => (text, GAME_PORT),
    };
    let fine = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_';
    if host.is_empty() || !host.chars().all(fine) {
        return Err(AddrError::Malformed);
    }
    port(number).map(|port| (host.to_ascii_lowercase(), port))
}

/// Returns the port, or an error for port 0.
fn port(number: u16) -> Result<u16, AddrError> {
    match number {
        0 => Err(AddrError::BadPort),
        number => Ok(number),
    }
}
