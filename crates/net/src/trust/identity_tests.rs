//! Tests of the server identity: made once, kept, never replaced, and its short key.

use std::path::PathBuf;

use super::{Identity, fingerprint, short_key};

/// Returns a fresh folder under the system's temporary folder.
fn scratch(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "abstractions-net-{name}-{}-{stamp}",
        std::process::id()
    ))
}

/// A second load of the same folder gives the same certificate and fingerprint.
#[test]
fn an_identity_is_made_once_and_kept() {
    let dir = scratch("identity");
    let first = Identity::load_or_create(&dir).unwrap();
    let again = Identity::load_or_create(&dir).unwrap();
    assert_eq!(first.cert(), again.cert());
    assert_eq!(first.fingerprint(), again.fingerprint());
    assert_eq!(first.fingerprint(), fingerprint(first.cert()));
    std::fs::remove_dir_all(dir).unwrap();
}

/// Both files are readable by their owner only.
#[cfg(unix)]
#[test]
fn the_key_files_are_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("private");
    Identity::load_or_create(&dir).unwrap();
    for file in ["cert.der", "key.der"] {
        let mode = std::fs::metadata(dir.join(file))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "{file}");
    }
    std::fs::remove_dir_all(dir).unwrap();
}

/// A folder with a certificate but no key is an error, not a reason to make a new identity.
#[test]
fn half_an_identity_is_refused() {
    let dir = scratch("half");
    Identity::load_or_create(&dir).unwrap();
    std::fs::remove_file(dir.join("key.der")).unwrap();
    assert!(Identity::load_or_create(&dir).is_err());
    assert!(!dir.join("key.der").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

/// The short key is the first six bytes in upper hex, in three groups.
#[test]
fn the_short_key_reads_in_three_groups() {
    let mut print = [0; 32];
    print[..6].copy_from_slice(&[0x7f, 0x3a, 0x91, 0xc2, 0x0b, 0x5e]);
    print[6] = 0xff;
    assert_eq!(short_key(&print), "7F3A-91C2-0B5E");
}
