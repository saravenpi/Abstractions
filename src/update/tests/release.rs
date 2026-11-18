//! Release tests: reading the document GitHub serves and picking the file of each platform.

use crate::update::check::check;
use crate::update::fake::Fake;
use crate::update::platform::{Platform, opener, pick};
use crate::update::release::{notes, parse};
use crate::update::{Failure, Status};

/// The release document of the GitHub API, cut down to a few fields and files, as recorded.
const RECORDED: &str = r###"{
  "url": "https://api.github.com/repos/saravenpi/Abstractions/releases/1",
  "tag_name": "v0.9.1",
  "name": "v0.9.1",
  "draft": false,
  "prerelease": false,
  "published_at": "2026-11-20T09:30:00Z",
  "author": { "login": "saravenpi", "id": 1 },
  "assets": [
    { "name": "abstractions-linux-x86_64.tar.gz", "size": 31000000, "state": "uploaded",
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/abstractions-linux-x86_64.tar.gz" },
    { "name": "abstractions-v0.9.1-linux-x86_64", "size": 88000000,
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/abstractions-v0.9.1-linux-x86_64" },
    { "name": "SHA256SUMS-linux-x86_64-v0.9.1.txt", "size": 300,
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/SHA256SUMS-linux-x86_64-v0.9.1.txt" },
    { "name": "abstractions-v0.9.1-windows-x86_64.exe", "size": 90000000,
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/abstractions-v0.9.1-windows-x86_64.exe" },
    { "name": "Abstractions-v0.9.1-macos-universal.dmg", "size": 120000000,
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/Abstractions-v0.9.1-macos-universal.dmg" },
    { "name": "SHA256SUMS-macos-universal-v0.9.1.txt", "size": 200,
      "browser_download_url":
      "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/SHA256SUMS-macos-universal-v0.9.1.txt" }
  ],
  "body": "## What's Changed\r\n* **Rivers** run to the sea by @saravenpi\r\n\r\n- Fixed `moons`\r\n### Notes\r\nThanks"
}"###;

/// The recorded document gives the version, the day, the notes as plain lines and the files.
#[test]
fn a_recorded_release_reads() {
    let release = parse(RECORDED.as_bytes()).unwrap();
    assert_eq!(release.version.to_string(), "0.9.1");
    assert_eq!(
        (release.tag.as_str(), release.date.as_str()),
        ("v0.9.1", "2026-11-20")
    );
    assert!(release.stable);
    assert_eq!(release.assets.len(), 6);
    assert_eq!(release.assets[1].size, 88_000_000);
    let wanted = [
        "What's Changed",
        "- Rivers run to the sea by @saravenpi",
        "- Fixed moons",
        "Notes",
        "Thanks",
    ];
    assert_eq!(release.notes, wanted);
}

/// Each platform gets its own plain file and checksum list, and nothing when one is missing.
#[test]
fn each_platform_gets_its_own_file() {
    let release = parse(RECORDED.as_bytes()).unwrap();
    let (payload, sums) = pick(&release, Platform::Linux).unwrap();
    assert_eq!(payload.name, "abstractions-v0.9.1-linux-x86_64");
    assert_eq!(sums.name, "SHA256SUMS-linux-x86_64-v0.9.1.txt");
    let (image, _) = pick(&release, Platform::Mac).unwrap();
    assert_eq!(image.name, "Abstractions-v0.9.1-macos-universal.dmg");
    assert_eq!(pick(&release, Platform::Windows), None, "no checksum list");
    let name = Platform::Windows.payload("v0.9.1");
    assert_eq!(name, "abstractions-v0.9.1-windows-x86_64.exe");
}

/// Releases have builds for three platforms, and each system has its opener.
#[test]
fn platforms_and_openers_are_known() {
    assert_eq!(Platform::of("linux", "x86_64"), Some(Platform::Linux));
    assert_eq!(Platform::of("windows", "x86_64"), Some(Platform::Windows));
    assert_eq!(Platform::of("macos", "aarch64"), Some(Platform::Mac));
    assert_eq!(Platform::of("linux", "aarch64"), None);
    assert_eq!(Platform::of("freebsd", "x86_64"), None);
    assert_eq!(opener("linux").0, "xdg-open");
    assert_eq!(opener("macos").0, "open");
    assert_eq!(opener("windows"), ("cmd", &["/C", "start", ""][..]));
}

/// A document that is not a release, or whose tag is no version, is unreadable.
#[test]
fn a_broken_document_is_unreadable() {
    for json in [
        "",
        "<html>",
        "{}",
        r#"{"tag_name":"nightly"}"#,
        r#"{"tag_name":7}"#,
    ] {
        assert_eq!(parse(json.as_bytes()), Err(Failure::Unreadable), "{json}");
    }
    let bare = parse(br#"{"tag_name":"v1.2.3"}"#).unwrap();
    assert!(bare.assets.is_empty() && bare.notes.is_empty() && bare.date.is_empty());
}

/// Long notes are cut to their first lines, and a long line to its first characters.
#[test]
fn long_notes_are_cut() {
    let body = format!("{}\n", "x".repeat(500)).repeat(200);
    let lines = notes(&body);
    assert_eq!(lines.len(), 60);
    assert!(lines.iter().all(|line| line.chars().count() == 160));
}

/// A check finds a newer release, and counts an older, equal, draft or pre-release one as
/// up to date.
#[test]
fn a_check_compares_with_the_running_version() {
    let fake = Fake::with_release("v0.9.1", Platform::Linux, b"new");
    let found = check(&fake, "0.9.0");
    assert!(matches!(found, Status::Available(release) if release.tag == "v0.9.1"));
    assert_eq!(check(&fake, "0.9.1"), Status::UpToDate);
    assert_eq!(check(&fake, "0.10.0"), Status::UpToDate);
    let early = Fake::with_release("v0.9.1-rc.1", Platform::Linux, b"new");
    assert_eq!(check(&early, "0.9.0"), Status::UpToDate);
    let marked = RECORDED.replace(r#""prerelease": false"#, r#""prerelease": true"#);
    assert!(!parse(marked.as_bytes()).unwrap().stable);
    assert_eq!(
        check(&Fake::default(), "0.9.0"),
        Status::Failed("No connection".into())
    );
}
