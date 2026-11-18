//! Tests of the trust-on-first-use check on real certificates.

use rustls::client::danger::ServerCertVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};

use super::{TrustError, Verifier, fingerprint};

/// Returns a fresh self-signed certificate.
fn cert() -> CertificateDer<'static> {
    let made = rcgen::generate_simple_self_signed(vec!["abstractions".to_string()]).unwrap();
    made.cert.der().clone()
}

/// Runs the check on `cert` and returns whether it accepted it.
fn accepts(verifier: &Verifier, cert: &CertificateDer<'_>) -> bool {
    let name = ServerName::try_from("abstractions").unwrap();
    verifier
        .verify_server_cert(cert, &[], &name, &[], UnixTime::now())
        .is_ok()
}

/// An unknown server is accepted and reported as new.
#[test]
fn an_unknown_server_is_accepted_the_first_time() {
    let cert = cert();
    let verifier = Verifier::new(None, None, true);
    assert!(accepts(&verifier, &cert));
    let seen = verifier.outcome().unwrap().unwrap();
    assert_eq!(seen.fingerprint, fingerprint(&cert));
    assert!(seen.first_time);
}

/// A known server with the same certificate is accepted and not new.
#[test]
fn a_known_server_is_accepted_again() {
    let cert = cert();
    let verifier = Verifier::new(Some(fingerprint(&cert)), None, true);
    assert!(accepts(&verifier, &cert));
    assert!(!verifier.outcome().unwrap().unwrap().first_time);
}

/// A known server with another certificate is refused, and the error names both keys.
#[test]
fn a_changed_key_is_refused() {
    let (old, new) = (cert(), cert());
    let verifier = Verifier::new(Some(fingerprint(&old)), None, true);
    assert!(!accepts(&verifier, &new));
    let error = verifier.outcome().unwrap().unwrap_err();
    let changed = TrustError::Changed {
        expected: fingerprint(&old),
        got: fingerprint(&new),
    };
    assert_eq!(error, changed);
}

/// A key announced by a beacon must match, even for a server never seen.
#[test]
fn an_announced_key_must_match() {
    let (announced, shown) = (cert(), cert());
    let verifier = Verifier::new(None, Some(fingerprint(&announced)), true);
    assert!(!accepts(&verifier, &shown));
    let good = Verifier::new(None, Some(fingerprint(&shown)), true);
    assert!(accepts(&good, &shown));
}

/// The check for status requests accepts a changed key and still reports it.
#[test]
fn a_status_check_only_reports() {
    let (old, new) = (cert(), cert());
    let verifier = Verifier::new(Some(fingerprint(&old)), None, false);
    assert!(accepts(&verifier, &new));
    assert_eq!(verifier.fingerprint(), Some(fingerprint(&new)));
}
