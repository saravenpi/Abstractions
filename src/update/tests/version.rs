//! Version tests: reading and comparing `major.minor.patch`.

use crate::update::version::{Version, newer, parse};

/// A plain version reads, with or without the `v`, and prints back without it.
#[test]
fn a_version_reads_with_or_without_the_prefix() {
    let wanted = Version {
        major: 0,
        minor: 9,
        patch: 1,
    };
    assert_eq!(parse("0.9.1"), Some(wanted));
    assert_eq!(parse("v0.9.1"), Some(wanted));
    assert_eq!(parse(" V0.9.1\n"), Some(wanted));
    assert_eq!(wanted.to_string(), "0.9.1");
}

/// A pre-release or build suffix is dropped: only the three numbers count.
#[test]
fn a_suffix_is_ignored() {
    assert_eq!(parse("v0.9.1-rc.2"), parse("0.9.1"));
    assert_eq!(parse("0.9.1+build.7"), parse("0.9.1"));
    assert!(!newer("0.9.1", "v0.9.1-rc.2"));
    assert!(newer("0.9.0", "v0.9.1-rc.2"));
}

/// Numbers compare as numbers, not as text: 0.10.0 is newer than 0.9.0.
#[test]
fn numbers_compare_as_numbers() {
    assert!(newer("0.9.0", "0.10.0"));
    assert!(!newer("0.10.0", "0.9.0"));
    assert!(newer("0.9.9", "0.10.0"));
    assert!(newer("0.9.0", "v0.9.1"));
    assert!(newer("0.9.12", "1.0.0"));
    assert!(newer("1.2.3", "1.2.10"));
    assert!(!newer("0.9.0", "0.9.0"));
    assert!(!newer("0.9.1", "v0.9.0"));
    assert!(!newer("1.0.0", "0.99.99"));
}

/// Garbage is no version, and is never newer than anything.
#[test]
fn garbage_is_no_version() {
    let garbage = [
        "",
        "v",
        "latest",
        "0.9",
        "0.9.1.4",
        "0..1",
        "a.b.c",
        "0.9.x",
        "-1.0.0",
        "0.9.+1",
        "1.0.0 beta",
        "0.9.99999999999999999999999",
    ];
    for text in garbage {
        assert_eq!(parse(text), None, "{text:?}");
        assert!(!newer("0.9.0", text), "{text:?}");
        assert!(!newer(text, "0.9.1"), "{text:?}");
    }
}
