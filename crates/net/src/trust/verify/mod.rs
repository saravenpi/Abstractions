//! The check a player runs on a server's certificate: its fingerprint instead of a chain.
//!
//! [`Verifier`] holds the check; `handshake` hands it to rustls.

mod handshake;

use std::sync::Mutex;

use rustls::client::danger::ServerCertVerified;
use rustls::crypto::WebPkiSupportedAlgorithms;
use rustls::pki_types::CertificateDer;
use rustls::{CertificateError, Error};

use super::{TrustError, fingerprint};

/// What a player learned of a server it accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    /// Fingerprint of the server's certificate.
    pub fingerprint: [u8; 32],
    /// True when the player had never seen this server: the caller should remember it.
    pub first_time: bool,
}

/// Trust on first use for one connection.
///
/// The chain and the name are not checked: the certificate's fingerprint must match the one
/// remembered for the address and the one a LAN beacon announced, when there are. The
/// handshake signatures are still checked with ring, so the server proves it holds the key.
#[derive(Debug)]
pub struct Verifier {
    /// Fingerprint remembered for this address, if the player saw it before.
    known: Option<[u8; 32]>,
    /// Fingerprint the caller expects, such as the one a LAN beacon carried.
    expected: Option<[u8; 32]>,
    /// When false, any certificate is accepted and only reported, for status requests.
    strict: bool,
    /// Fingerprint of the certificate the server showed, once it did.
    seen: Mutex<Option<[u8; 32]>>,
    /// Signature algorithms of ring, for the handshake signatures.
    algorithms: WebPkiSupportedAlgorithms,
}

impl Verifier {
    /// Makes a check for one connection. With `strict` false it refuses nothing.
    pub fn new(known: Option<[u8; 32]>, expected: Option<[u8; 32]>, strict: bool) -> Self {
        Self {
            known,
            expected,
            strict,
            seen: Mutex::new(None),
            algorithms: rustls::crypto::ring::default_provider().signature_verification_algorithms,
        }
    }

    /// Judges a fingerprint: returns whether it is new, or why it is refused.
    fn judge(&self, got: [u8; 32]) -> Result<bool, TrustError> {
        for expected in [self.expected, self.known].into_iter().flatten() {
            if expected != got {
                return Err(TrustError::Changed { expected, got });
            }
        }
        Ok(self.known.is_none())
    }

    /// Returns the fingerprint the server showed, once the handshake reached it.
    pub fn fingerprint(&self) -> Option<[u8; 32]> {
        *self
            .seen
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }

    /// Remembers the fingerprint of `end_entity` and accepts it when it is the one expected or
    /// a new one. A changed one is refused when strict, else accepted and only reported.
    fn check(&self, end_entity: &CertificateDer<'_>) -> Result<ServerCertVerified, Error> {
        let got = fingerprint(end_entity);
        *self
            .seen
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(got);
        match self.judge(got) {
            Err(_) if self.strict => Err(Error::InvalidCertificate(
                CertificateError::ApplicationVerificationFailure,
            )),
            _ => Ok(ServerCertVerified::assertion()),
        }
    }

    /// Returns the verdict on the certificate the server showed, once it did.
    pub fn outcome(&self) -> Option<Result<Seen, TrustError>> {
        let got = self.fingerprint()?;
        Some(self.judge(got).map(|first_time| Seen {
            fingerprint: got,
            first_time,
        }))
    }
}
