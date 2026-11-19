//! The verifier as rustls calls it: the certificate check, and the handshake signatures
//! checked with ring.

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};

use super::Verifier;

/// Writes the `verify_server_cert` method of [`ServerCertVerifier`] as a call to the method
/// `$check` of the verifier with the certificate alone. Rustls fixes six parameters, but the
/// chain, the name, the OCSP answer and the time play no part in trust on first use.
macro_rules! verify_server_cert_by {
    ($check:ident) => {
        /// Accepts the certificate when its fingerprint is the one expected, or a new one.
        fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &ServerName<'_>,
            _ocsp_response: &[u8],
            _now: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            self.$check(end_entity)
        }
    };
}

impl ServerCertVerifier for Verifier {
    verify_server_cert_by!(check);

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
