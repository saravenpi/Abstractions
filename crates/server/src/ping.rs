//! `abstractions-server ping <address>`: asks a server what it is.

use std::process::ExitCode;
use std::time::Duration;

use abstractions_net::io::status;
use abstractions_net::trust::short_key;
use abstractions_net::wire::{PROTOCOL, Status};

/// Longest wait for the answer.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Asks the server at `address` for its status and prints it. Fails when it does not answer.
pub fn ping(address: &str) -> ExitCode {
    match status(address, TIMEOUT) {
        Ok((status, rtt, fingerprint)) => {
            println!("{}", line(&status, rtt, &short_key(&fingerprint)));
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("{}", reason.words(address, None));
            ExitCode::FAILURE
        }
    }
}

/// Returns the one line a status prints as, with a note when the protocols differ.
pub fn line(status: &Status, rtt: Duration, key: &str) -> String {
    let mut text = format!(
        "{}   {}/{} players   v{}   {} ms   key {key}",
        status.name,
        status.players,
        status.cap,
        status.version,
        rtt.as_millis()
    );
    if status.protocol != PROTOCOL {
        text += &format!(
            "   protocol {}, this program speaks {PROTOCOL}",
            status.protocol
        );
    }
    text
}
