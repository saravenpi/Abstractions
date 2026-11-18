//! File tests: checksums, the two ways of swapping an executable, and read-only folders.

use std::fs;

use super::scratch;
use crate::update::Failure;
use crate::update::checksum::{digest, listed, verify};
use crate::update::fixture::sha256;
use crate::update::install::{swap_aside, swap_over, sweep};
use crate::update::place::{old_path, part_path, writable};

/// SHA-256 of the three letters `abc`, from the standard's own examples.
const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// A file hashes to the known value, and a checksum list is read by file name.
#[test]
fn a_checksum_list_is_read_by_name() {
    let dir = scratch("sums");
    fs::write(dir.join("game"), "abc").unwrap();
    assert_eq!(digest(&dir.join("game")).unwrap(), ABC);
    assert_eq!(sha256(b"abc"), ABC);
    let upper = ABC.to_uppercase();
    let list = format!(
        "{}  other\n{upper} *game.exe\n{ABC}  game\nshort  bad\n",
        "0".repeat(64)
    );
    assert_eq!(listed(&list, "game").as_deref(), Some(ABC));
    assert_eq!(listed(&list, "game.exe").as_deref(), Some(ABC));
    assert_eq!(listed(&list, "bad"), None);
    assert_eq!(listed(&list, "missing"), None);
    assert_eq!(listed("", "game"), None);
}

/// Good data passes; corrupt data, a truncated file and an unlisted file all fail.
#[test]
fn corrupt_data_fails_the_checksum() {
    let dir = scratch("verify");
    let file = dir.join("game");
    let list = format!("{ABC}  game\n");
    fs::write(&file, "abc").unwrap();
    assert_eq!(verify(&file, &list, "game"), Ok(()));
    fs::write(&file, "abd").unwrap();
    assert_eq!(verify(&file, &list, "game"), Err(Failure::Checksum));
    fs::write(&file, "ab").unwrap();
    assert_eq!(verify(&file, &list, "game"), Err(Failure::Checksum));
    fs::write(&file, "abc").unwrap();
    assert_eq!(verify(&file, &list, "other"), Err(Failure::Checksum));
    assert_eq!(verify(&file, "", "game"), Err(Failure::Checksum));
    assert!(matches!(
        verify(&dir.join("gone"), &list, "game"),
        Err(Failure::Disk(_))
    ));
}

/// The names beside the executable keep its extension last.
#[test]
fn the_side_files_are_named_after_the_executable() {
    let dir = scratch("names");
    assert_eq!(
        old_path(&dir.join("abstractions")),
        dir.join("abstractions.old")
    );
    assert_eq!(
        old_path(&dir.join("abstractions.exe")),
        dir.join("abstractions.old.exe")
    );
    assert_eq!(
        part_path(&dir.join("abstractions")),
        dir.join("abstractions.part")
    );
    assert_eq!(
        part_path(&dir.join("abstractions.exe")),
        dir.join("abstractions.part.exe")
    );
    assert_eq!(
        part_path(&dir.join("Game-v1.dmg")),
        dir.join("Game-v1.part.dmg")
    );
}

/// The Linux way: the new file takes the name, and the old one is kept beside it.
#[test]
fn the_linux_swap_keeps_the_old_game() {
    let dir = scratch("swap-over");
    let (exe, new) = (dir.join("abstractions"), dir.join("abstractions.part"));
    fs::write(&exe, "old").unwrap();
    fs::write(&new, "new").unwrap();
    fs::write(old_path(&exe), "older").unwrap();
    swap_over(&new, &exe).unwrap();
    assert_eq!(fs::read_to_string(&exe).unwrap(), "new");
    assert_eq!(fs::read_to_string(old_path(&exe)).unwrap(), "old");
    assert!(!new.exists());
    assert!(swap_over(&dir.join("gone"), &exe).is_err());
    assert_eq!(
        fs::read_to_string(&exe).unwrap(),
        "new",
        "a failed swap changes nothing"
    );
    assert!(!old_path(&exe).exists());
}

/// The Windows way: the old file moves aside, the new one takes its name, and a failure of the
/// second step moves the old file back.
#[test]
fn the_windows_swap_moves_the_old_game_aside() {
    let dir = scratch("swap-aside");
    let (exe, new) = (
        dir.join("abstractions.exe"),
        dir.join("abstractions.part.exe"),
    );
    fs::write(&exe, "old").unwrap();
    fs::write(&new, "new").unwrap();
    swap_aside(&new, &exe).unwrap();
    assert_eq!(fs::read_to_string(&exe).unwrap(), "new");
    assert_eq!(
        fs::read_to_string(dir.join("abstractions.old.exe")).unwrap(),
        "old"
    );
    assert!(!new.exists());
    assert!(swap_aside(&dir.join("gone.exe"), &exe).is_err());
    assert_eq!(
        fs::read_to_string(&exe).unwrap(),
        "new",
        "the old file moved back"
    );
    assert!(!old_path(&exe).exists());
}

/// The next start removes the old game and any unfinished download, and nothing else.
#[test]
fn a_start_sweeps_what_an_update_left() {
    let dir = scratch("sweep");
    let exe = dir.join("abstractions");
    for name in [
        "abstractions",
        "abstractions.old",
        "abstractions.part",
        "save.dat",
    ] {
        fs::write(dir.join(name), "x").unwrap();
    }
    sweep(&exe);
    assert!(exe.exists() && dir.join("save.dat").exists());
    assert!(!old_path(&exe).exists() && !part_path(&exe).exists());
    sweep(&exe);
    sweep(std::path::Path::new(""));
}

/// A folder that is missing, is a file, or is read-only is not writable.
#[test]
fn a_read_only_folder_is_detected() {
    let dir = scratch("writable");
    assert!(writable(&dir));
    assert!(
        fs::read_dir(&dir).unwrap().count() == 1,
        "the probe is removed"
    );
    assert!(!writable(&dir.join("missing")));
    fs::write(dir.join("file"), "x").unwrap();
    assert!(!writable(&dir.join("file")));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let locked = dir.join("locked");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
        let root = fs::File::create(locked.join("probe")).is_ok();
        assert_eq!(
            writable(&locked),
            root,
            "only root writes to a read-only folder"
        );
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    }
}
