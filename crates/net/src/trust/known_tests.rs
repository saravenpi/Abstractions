//! Tests of the list of known servers: missing, saved, read back and damaged files.

use std::path::PathBuf;

use super::KnownServers;

/// Returns a path for a list under a fresh temporary folder.
fn scratch(name: &str) -> PathBuf {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir()
        .join(format!(
            "abstractions-net-{name}-{}-{stamp}",
            std::process::id()
        ))
        .join("known_servers.toml")
}

/// A missing file is an empty list.
#[test]
fn a_missing_list_is_empty() {
    let known = KnownServers::load(scratch("missing")).unwrap();
    assert_eq!(known.get("example.org:24650"), None);
}

/// What is saved is read back, address by address, with IPv6 addresses kept whole.
#[test]
fn a_saved_list_reads_back() {
    let path = scratch("saved");
    let mut known = KnownServers::load(&path).unwrap();
    known.insert("[::1]:24650", [7; 32]);
    known.insert("play.example.org:24650", [0xab; 32]);
    known.save().unwrap();
    let again = KnownServers::load(&path).unwrap();
    assert_eq!(again, known);
    assert_eq!(again.get("[::1]:24650"), Some([7; 32]));
    assert_eq!(again.get("play.example.org:24650"), Some([0xab; 32]));
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

/// A damaged file is an error, so trusted keys are never dropped without a word.
#[test]
fn a_damaged_list_is_an_error() {
    let path = scratch("damaged");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "not = [toml").unwrap();
    assert!(KnownServers::load(&path).is_err());
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

/// A fingerprint that is not 64 hex digits is ignored.
#[test]
fn a_bad_fingerprint_is_ignored() {
    let path = scratch("bad");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "\"a:1\" = \"abc\"\n").unwrap();
    assert_eq!(KnownServers::load(&path).unwrap().get("a:1"), None);
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
