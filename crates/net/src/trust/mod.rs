//! Trust on first use, like SSH: the certificate a server makes for itself, its fingerprint and
//! the list of servers a player already knows.
//!
//! A server makes a self-signed certificate on its first start and keeps it ([`Identity`]). A
//! player remembers the SHA-256 of that certificate per address ([`KnownServers`]) and refuses a
//! server whose certificate changed ([`Verifier`]). No web PKI is involved: a domain is only a
//! name for an address.

mod identity;
mod known;
mod verify;

#[cfg(test)]
mod identity_tests;
#[cfg(test)]
mod known_tests;
#[cfg(test)]
mod verify_tests;

pub use identity::{Identity, SERVER_NAME};
pub use known::KnownServers;
pub use verify::{Seen, Verifier};

use std::fmt;

use sha2::{Digest, Sha256};

/// Returns the SHA-256 of a certificate in DER form: the key a player remembers.
pub fn fingerprint(cert_der: &[u8]) -> [u8; 32] {
    Sha256::digest(cert_der).into()
}

/// Returns the short key shown to humans: the first 6 bytes of a fingerprint, such as
/// `7F3A-91C2-0B5E`.
pub fn short_key(fingerprint: &[u8; 32]) -> String {
    fingerprint[..6]
        .chunks(2)
        .map(|pair| format!("{:02X}{:02X}", pair[0], pair[1]))
        .collect::<Vec<_>>()
        .join("-")
}

/// Why a player does not trust a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustError {
    /// The server shows another certificate than the one remembered or announced.
    Changed {
        /// Fingerprint the player expected.
        expected: [u8; 32],
        /// Fingerprint the server showed.
        got: [u8; 32],
    },
    /// The list of known servers could not be read or written.
    Store(String),
}

impl fmt::Display for TrustError {
    /// Writes the error in plain words, ready for the screen.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Changed { expected, got } => write!(
                f,
                "This server's key changed: expected {}, got {}",
                short_key(expected),
                short_key(got)
            ),
            Self::Store(error) => write!(f, "The list of known servers is unusable: {error}"),
        }
    }
}

impl std::error::Error for TrustError {}
