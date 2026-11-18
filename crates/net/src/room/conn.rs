//! A connection as the room sees it, and the counters that keep it from flooding.

use std::net::IpAddr;

/// Reliable messages a connection may send per second before it is closed.
pub(super) const MESSAGES_PER_SECOND: u32 = 60;

/// Log entries a player may add per second. Further ones are dropped.
pub(super) const CHANGES_PER_SECOND: u32 = 10;

/// Seconds a connection may stay open without joining.
pub(super) const HELLO_WAIT: f64 = 10.0;

/// Most open connections from one IP address.
pub(super) const CONNS_PER_IP: usize = 8;

/// A counter of events over the current one-second window.
#[derive(Debug, Clone, Copy)]
pub(super) struct Window {
    /// Network time the window opened.
    start: f64,
    /// Events counted in the window.
    count: u32,
}

impl Window {
    /// Returns a window opened at `now` with nothing counted.
    pub(super) fn new(now: f64) -> Self {
        Self {
            start: now,
            count: 0,
        }
    }

    /// Counts one event at `now` and returns true while the window holds at most `max`. A new
    /// window opens once the current one is a second old.
    pub(super) fn allow(&mut self, now: f64, max: u32) -> bool {
        if now - self.start >= 1.0 {
            *self = Self::new(now);
        }
        self.count += 1;
        self.count <= max
    }
}

/// One open connection.
#[derive(Debug)]
pub(super) struct Conn {
    /// Address the connection came from.
    pub(super) ip: IpAddr,
    /// Network time the connection opened.
    pub(super) opened: f64,
    /// The seat of the player, once joined.
    pub(super) seat: Option<usize>,
    /// Reliable messages this second.
    pub(super) messages: Window,
    /// Log entries this second.
    pub(super) changes: Window,
}

impl Conn {
    /// Returns a connection from `ip` opened at `now`, not joined yet.
    pub(super) fn new(ip: IpAddr, now: f64) -> Self {
        Self {
            ip,
            opened: now,
            seat: None,
            messages: Window::new(now),
            changes: Window::new(now),
        }
    }
}
