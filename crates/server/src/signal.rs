//! Waiting for the owner to stop the server: SIGTERM or SIGINT, or Ctrl-C on Windows.

use std::io;

use async_signal::{Signal, Signals};
use smol::stream::StreamExt;

/// Signals that stop the server.
#[cfg(unix)]
const STOP: [Signal; 2] = [Signal::Term, Signal::Int];

/// Signals that stop the server. Windows only knows Ctrl-C.
#[cfg(not(unix))]
const STOP: [Signal; 1] = [Signal::Int];

/// Starts catching the stop signals. Call it before the server starts, so none is missed.
pub fn listen() -> io::Result<Signals> {
    Signals::new(STOP)
}

/// Blocks until a stop signal arrives.
pub fn wait(signals: &mut Signals) {
    smol::block_on(signals.next());
}
