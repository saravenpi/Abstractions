//! The check a player runs on a server's certificate: its fingerprint instead of a chain.

use std::sync::Mutex;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{WebPkiSupportedAlgorithms, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{CertificateError, DigitallySignedStruct, Error, SignatureScheme};

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

    /// Returns the verdict on the certificate the server showed, once it did.
    pub fn outcome(&self) -> Option<Result<Seen, TrustError>> {
        let got = self.fingerprint()?;
        Some(self.judge(got).map(|first_time| Seen {
            fingerprint: got,
            first_time,
        }))
    }
}

impl ServerCertVerifier for Verifier {
    /// Accepts the certificate when its fingerprint is the one expected, or a new one.
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
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

    /// Checks a TLS 1.2 handshake signature with ring. QUIC never uses TLS 1.2.
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls12_signature(message, cert, dss, &self.algorithms)
    }

    /// Checks the TLS 1.3 handshake signature with ring: the server holds the certificate's key.
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify_tls13_signature(message, cert, dss, &self.algorithms)
    }

    /// Returns the signature schemes ring can check.
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algorithms.supported_schemes()
    }
}
