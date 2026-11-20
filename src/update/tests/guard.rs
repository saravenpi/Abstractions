//! Guard tests: which addresses may be fetched, and what a download refuses.

use std::sync::Arc;

use super::{payload, scratch};
use crate::update::allow::{self, RELEASES};
use crate::update::download::{CAP, download};
use crate::update::fake::Fake;
use crate::update::{Asset, Failure, Signals, UpdatePrefs};

/// Only files under the game's own release page are downloaded.
#[test]
fn only_the_release_page_is_downloaded_from() {
    let good = format!("{RELEASES}v0.9.1/abstractions-v0.9.1-linux-x86_64");
    assert!(allow::download(&good));
    let bad = [
        "http://github.com/saravenpi/Abstractions/releases/download/v0.9.1/a",
        "https://github.com.evil.example/saravenpi/Abstractions/releases/download/v0.9.1/a",
        "https://evil.example/https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/a",
        "https://github.com/saravenpi/Abstractions/releases/download.evil.example/a",
        "https://github.com/saravenpi/Abstractions/releases/downloads/v0.9.1/a",
        "https://github.com/saravenpi/Abstractions-fork/releases/download/v0.9.1/a",
        "https://github.com/someone/Abstractions/releases/download/v0.9.1/a",
        "https://github.com/saravenpi/Abstractions/releases/download/../../../../evil/a",
        "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/a?x=@evil.example",
        "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/a b",
        "https://github.com/saravenpi/Abstractions/releases/download/",
        "https://user@github.com/saravenpi/Abstractions/releases/download/v0.9.1/a",
        "",
    ];
    for url in bad {
        assert!(!allow::download(url), "{url}");
    }
}

/// A redirect is followed only over HTTPS, to GitHub or one of its file hosts.
#[test]
fn redirects_stay_on_github_hosts() {
    let good = [
        "https://release-assets.githubusercontent.com/github-production-release-asset/1/2?sig=abc",
        "https://objects.githubusercontent.com/github-production-release-asset-2e65be/1",
        "https://github.com/saravenpi/Abstractions/releases/download/v0.9.1/a",
    ];
    for url in good {
        assert!(allow::redirect(url), "{url}");
    }
    let bad = [
        "http://release-assets.githubusercontent.com/a",
        "https://githubusercontent.com.evil.example/a",
        "https://evilgithubusercontent.com/a",
        "https://github.com.evil.example/a",
        "https://github.com@evil.example/a",
        "https://github.com:8443/a",
        "https://evil.example/?https://github.com/",
        "/saravenpi/Abstractions/releases/download/v0.9.1/a",
        "ftp://github.com/a",
        "",
    ];
    for url in bad {
        assert!(!allow::redirect(url), "{url}");
    }
}

/// Returns a release file of `size` bytes at `url`.
fn asset(url: &str, size: u64) -> Asset {
    Asset {
        name: "game".into(),
        url: url.into(),
        size,
    }
}

/// A file from elsewhere, an empty one or one over the cap is refused before any request.
#[test]
fn a_download_is_refused_before_any_request() {
    let dir = scratch("refused");
    let (fake, dest) = (Fake::default(), dir.join("game.part"));
    let signals = Signals::default();
    let good = format!("{RELEASES}v0.9.1/game");
    let elsewhere = asset("https://evil.example/game", 5000);
    assert_eq!(
        download(&fake, &elsewhere, &dest, &signals),
        Err(Failure::Forbidden)
    );
    let huge = asset(&good, CAP + 1);
    assert_eq!(
        download(&fake, &huge, &dest, &signals),
        Err(Failure::TooLarge)
    );
    assert_eq!(
        download(&fake, &asset(&good, 0), &dest, &signals),
        Err(Failure::TooLarge)
    );
    assert!(fake.asked.lock().unwrap().is_empty());
    assert!(!dest.exists());
}

/// A download shorter or longer than the release declares is removed.
#[test]
fn a_download_of_the_wrong_size_is_removed() {
    let dir = scratch("size");
    let (fake, dest) = (Fake::default(), dir.join("game.part"));
    let url = format!("{RELEASES}v0.9.1/game");
    fake.serve(&url, &payload());
    for declared in [4999, 5001, 1024, 10_000] {
        let signals = Signals::default();
        let done = download(&fake, &asset(&url, declared), &dest, &signals);
        assert_eq!(done, Err(Failure::Size), "{declared}");
        assert!(!dest.exists(), "{declared}");
    }
    let signals = Signals::default();
    assert_eq!(download(&fake, &asset(&url, 5000), &dest, &signals), Ok(()));
    assert_eq!(
        (std::fs::read(&dest).unwrap(), signals.done()),
        (payload(), 5000)
    );
}

/// A cancel in the middle of a download stops it and removes the file.
#[test]
fn a_cancelled_download_leaves_no_file() {
    let dir = scratch("cancel-download");
    let (fake, dest) = (Arc::new(Fake::default()), dir.join("game.part"));
    let url = format!("{RELEASES}v0.9.1/game");
    fake.serve(&url, &payload());
    fake.hold.store(true, std::sync::atomic::Ordering::SeqCst);
    let signals = Arc::new(Signals::default());
    let (host, shared, to) = (fake.clone(), signals.clone(), dest.clone());
    let thread = std::thread::spawn(move || download(&*host, &asset(&url, 5000), &to, &shared));
    while signals.done() == 0 {
        std::thread::yield_now();
    }
    assert!(dest.exists());
    assert!(signals.cancel());
    fake.hold.store(false, std::sync::atomic::Ordering::SeqCst);
    assert_eq!(thread.join().unwrap(), Err(Failure::Cancelled));
    assert!(!dest.exists());
    assert!(!signals.commit(), "a cancelled install never commits");
}

/// A settings file written before the option existed loads with the check at launch on.
#[test]
fn old_settings_check_at_launch() {
    let old: crate::settings::Settings = toml::from_str("[graphics]\nfov = 80\n").unwrap();
    assert!(old.updates.check_at_launch);
    let empty: UpdatePrefs = toml::from_str("").unwrap();
    assert!(empty.check_at_launch);
    let off: crate::settings::Settings =
        toml::from_str("[updates]\ncheck_at_launch = false\n").unwrap();
    assert!(!off.updates.check_at_launch);
    let text = toml::to_string(&off).unwrap();
    assert_eq!(
        toml::from_str::<crate::settings::Settings>(&text).unwrap(),
        off
    );
}
