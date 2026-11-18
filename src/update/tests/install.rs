//! Install tests: the whole path on scratch folders, for each platform, with its failures.

use std::fs;
use std::sync::atomic::Ordering;

use super::{payload, settle, updater, wait};
use crate::update::allow::RELEASES;
use crate::update::install::sweep;
use crate::update::place::{old_path, part_path};
use crate::update::{Outcome, Platform, Stage, Status, Updates};

/// Checks, then installs, and waits for the end.
fn run(updates: &mut Updates) {
    updates.check();
    settle(updates);
    updates.install();
    assert_eq!(updates.stage(), &Stage::Downloading);
    settle(updates);
}

/// On Linux the new game takes the place of the old one, which is kept until the next start;
/// a restart starts the same path with the same arguments.
#[test]
fn an_install_swaps_the_executable_on_linux() {
    let (mut updates, fake, dir) = updater("linux", Platform::Linux, "v0.9.1");
    let exe = dir.join("abstractions");
    assert!(updates.restart().is_err(), "nothing to restart into yet");
    run(&mut updates);
    assert_eq!(updates.stage(), &Stage::Ready(Outcome::Swapped));
    assert_eq!(fs::read(&exe).unwrap(), payload());
    assert_eq!(fs::read_to_string(old_path(&exe)).unwrap(), "old");
    assert!(!part_path(&exe).exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&exe).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "the new game can be run");
    }
    updates.restart().unwrap();
    let launched = fake.launched.lock().unwrap().clone();
    assert_eq!(
        launched,
        [(exe.clone(), vec!["--seed".to_string(), "7".to_string()])]
    );
    updates.install();
    updates.check();
    assert_eq!(
        updates.stage(),
        &Stage::Ready(Outcome::Swapped),
        "one install per launch"
    );
    sweep(&exe);
    assert!(!old_path(&exe).exists() && exe.exists());
}

/// On Windows the running file moves aside as `abstractions.old.exe`.
#[test]
fn an_install_moves_the_executable_aside_on_windows() {
    let (mut updates, _, dir) = updater("windows", Platform::Windows, "v0.9.1");
    run(&mut updates);
    assert_eq!(updates.stage(), &Stage::Ready(Outcome::Swapped));
    assert_eq!(fs::read(dir.join("abstractions.exe")).unwrap(), payload());
    assert_eq!(
        fs::read_to_string(dir.join("abstractions.old.exe")).unwrap(),
        "old"
    );
    assert!(!dir.join("abstractions.part.exe").exists());
}

/// On macOS the disk image lands in Downloads, the application is untouched, and Open hands the
/// image to the system.
#[test]
fn an_install_leaves_the_disk_image_in_downloads_on_macos() {
    let (mut updates, fake, dir) = updater("macos", Platform::Mac, "v0.9.1");
    run(&mut updates);
    let image = dir.join("Downloads/Abstractions-v0.9.1-macos-universal.dmg");
    assert_eq!(
        updates.stage(),
        &Stage::Ready(Outcome::Image(image.clone()))
    );
    assert_eq!(fs::read(&image).unwrap(), payload());
    assert_eq!(fs::read_to_string(dir.join("abstractions")).unwrap(), "old");
    assert_eq!(fs::read_dir(dir.join("Downloads")).unwrap().count(), 1);
    assert!(
        updates.restart().is_err(),
        "the application was not replaced"
    );
    updates.open_image().unwrap();
    let launched = fake.launched.lock().unwrap().clone();
    assert_eq!(
        launched[0].1.last(),
        Some(&image.to_string_lossy().to_string())
    );
}

/// A damaged download, a missing checksum list and a list from elsewhere each fail with a
/// reason, remove the download and leave the installed game as it was. A retry can succeed.
#[test]
fn a_failed_install_leaves_the_game_untouched() {
    let (mut updates, fake, dir) = updater("corrupt", Platform::Linux, "v0.9.1");
    let exe = dir.join("abstractions");
    let file = format!("{RELEASES}v0.9.1/abstractions-v0.9.1-linux-x86_64");
    let sums = format!("{RELEASES}v0.9.1/SHA256SUMS-linux-x86_64-v0.9.1.txt");
    let mut damaged = payload();
    damaged[4000] ^= 1;
    fake.serve(&file, &damaged);
    run(&mut updates);
    let reason = "The download is damaged: its checksum does not match";
    assert_eq!(updates.stage(), &Stage::Failed(reason.into()));
    fake.serve(&file, &payload());
    let list = fake.files.lock().unwrap().remove(&sums).unwrap();
    updates.install();
    settle(&mut updates);
    assert_eq!(updates.stage(), &Stage::Failed("No connection".into()));
    assert_eq!(fs::read_to_string(&exe).unwrap(), "old");
    assert_eq!(
        fs::read_dir(&dir).unwrap().count(),
        2,
        "the game and Downloads only"
    );
    assert!(matches!(updates.status(), Status::Available(_)));
    fake.serve(&sums, &list);
    updates.install();
    settle(&mut updates);
    assert_eq!(updates.stage(), &Stage::Ready(Outcome::Swapped));
}

/// A cancel during the download returns to idle at once, leaves no temporary file and does not
/// touch the game; the late report of the cancelled thread changes nothing.
#[test]
fn a_cancel_during_the_download_leaves_no_temporary_file() {
    let (mut updates, fake, dir) = updater("cancel", Platform::Linux, "v0.9.1");
    let exe = dir.join("abstractions");
    updates.check();
    settle(&mut updates);
    fake.hold.store(true, Ordering::SeqCst);
    updates.install();
    wait(&mut updates, |updates| updates.progress().0 > 0);
    assert_eq!(updates.progress(), (1024, 5000));
    assert!(part_path(&exe).exists());
    updates.cancel();
    assert_eq!(updates.stage(), &Stage::Idle);
    fake.hold.store(false, Ordering::SeqCst);
    wait(&mut updates, |_| !part_path(&exe).exists());
    updates.poll();
    assert_eq!(updates.stage(), &Stage::Idle);
    assert_eq!(fs::read_to_string(&exe).unwrap(), "old");
    assert!(!old_path(&exe).exists());
    updates.install();
    settle(&mut updates);
    assert_eq!(
        updates.stage(),
        &Stage::Ready(Outcome::Swapped),
        "a new install still works"
    );
}
