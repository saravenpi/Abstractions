//! Updater tests. None touches the network: they use the fake host and scratch folders.
//! This file holds what they share; the tests are in the modules below.

mod files;
mod guard;
mod install;
mod machine;
mod release;
mod version;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::fake::Fake;
use super::{Place, Platform, Stage, Status, Updates};

/// The bytes of a made-up new game: long enough to arrive in several chunks.
pub(super) fn payload() -> Vec<u8> {
    (0..5000u32).map(|index| (index % 251) as u8).collect()
}

/// Returns an empty scratch folder of its own for one test.
pub(super) fn scratch(tag: &str) -> PathBuf {
    let name = format!("abstractions-update-{tag}-{}", std::process::id());
    let dir = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Downloads")).unwrap();
    dir
}

/// Returns a game of `platform` installed in `dir`, whose executable holds the text `old`.
pub(super) fn place(dir: &Path, platform: Platform) -> Place {
    let name = match platform {
        Platform::Windows => "abstractions.exe",
        _ => "abstractions",
    };
    std::fs::write(dir.join(name), "old").unwrap();
    Place {
        platform: Some(platform),
        exe: dir.join(name),
        downloads: Some(dir.join("Downloads")),
        args: vec!["--seed".into(), "7".into()],
    }
}

/// Returns an updater for version 0.9.0 of `platform` in a fresh folder, and the host behind it,
/// whose latest release is `tag`.
pub(super) fn updater(name: &str, platform: Platform, tag: &str) -> (Updates, Arc<Fake>, PathBuf) {
    let dir = scratch(name);
    let fake = Arc::new(Fake::with_release(tag, platform, &payload()));
    let updates = Updates::new(fake.clone(), place(&dir, platform), "0.9.0");
    (updates, fake, dir)
}

/// Waits, for a few seconds at most, until `done` holds, reading the thread reports meanwhile.
pub(super) fn wait(updates: &mut Updates, done: impl Fn(&Updates) -> bool) {
    for _ in 0..5000 {
        updates.poll();
        if done(updates) {
            return;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("the updater never got there");
}

/// Waits until no check and no download runs.
pub(super) fn settle(updates: &mut Updates) {
    wait(updates, |updates| {
        *updates.status() != Status::Checking && *updates.stage() != Stage::Downloading
    });
}
