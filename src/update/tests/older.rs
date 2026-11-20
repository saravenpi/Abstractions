//! Tests of "Open with 0.10": the whole path on the fake host and a scratch folder.

use std::fs;

use super::{payload, scratch, wait};
use crate::update::check::LATEST;
use crate::update::fake::Fake;
use crate::update::older::{OLDER_TAG, Older, TAGS, folder};
use crate::update::{Failure, Platform, Updates};

/// Returns a Linux game, the host that serves the 0.10 release with `body` as its build, and
/// the folder of the game's data.
fn setup(name: &str, body: &[u8]) -> (Updates, std::sync::Arc<Fake>, std::path::PathBuf) {
    let dir = scratch(name);
    let fake = Fake::with_release(OLDER_TAG, Platform::Linux, &payload());
    let release = fake.files.lock().unwrap().get(LATEST).cloned().unwrap();
    fake.serve(&format!("{TAGS}{OLDER_TAG}"), &release);
    let build = Platform::Linux.payload(OLDER_TAG);
    fake.serve(
        &format!("{}{OLDER_TAG}/{build}", crate::update::allow::RELEASES),
        body,
    );
    let fake = std::sync::Arc::new(fake);
    let updates = Updates::new(fake.clone(), super::place(&dir, Platform::Linux), "0.11.0");
    (updates, fake, dir)
}

/// Waits until the older game was started or failed.
fn finish(updates: &mut Updates) {
    wait(updates, |updates| *updates.older() != Older::Working);
}

/// The build is downloaded, checked, kept in the data folder and started with `--world`, and a
/// second time it is not downloaded again.
#[test]
fn a_world_opens_with_the_older_build() {
    let (mut updates, fake, dir) = setup("older", &payload());
    updates.open_older(OLDER_TAG, "reef", &dir);
    assert_eq!(updates.older(), &Older::Working);
    finish(&mut updates);
    assert_eq!(updates.older(), &Older::Opened);
    let build = folder(&dir, OLDER_TAG).join(Platform::Linux.payload(OLDER_TAG));
    assert_eq!(fs::read(&build).unwrap(), payload());
    let args = vec!["--world".to_string(), "reef".to_string()];
    assert_eq!(
        fake.launched.lock().unwrap().clone(),
        [(build.clone(), args)]
    );
    let url = format!(
        "{}{OLDER_TAG}/{}",
        crate::update::allow::RELEASES,
        Platform::Linux.payload(OLDER_TAG)
    );
    updates.open_older(OLDER_TAG, "isle", &dir);
    finish(&mut updates);
    assert_eq!(
        fake.count(&url),
        1,
        "the kept build is not downloaded again"
    );
    assert_eq!(fake.launched.lock().unwrap().len(), 2);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&build).unwrap().permissions().mode() & 0o111,
            0o111
        );
    }
}

/// A build whose checksum does not match is removed and never started.
#[test]
fn a_damaged_older_build_is_never_started() {
    let mut damaged = payload();
    damaged[10] ^= 0xff;
    let (mut updates, fake, dir) = setup("older-bad", &damaged);
    updates.open_older(OLDER_TAG, "reef", &dir);
    finish(&mut updates);
    assert_eq!(updates.older(), &Older::Failed(Failure::Checksum.reason()));
    assert!(fake.launched.lock().unwrap().is_empty());
    let kept = fs::read_dir(folder(&dir, OLDER_TAG)).unwrap().count();
    assert_eq!(kept, 0);
}
