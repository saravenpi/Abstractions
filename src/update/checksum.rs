//! SHA-256 of a downloaded file, checked against the checksum list of its release.
//!
//! This proves the download is whole. It does not prove who made it: the list comes from the
//! same release, over the same HTTPS connection to GitHub.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::failure::Failure;

/// Returns the SHA-256 of a file as lowercase hexadecimal.
pub fn digest(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut chunk = vec![0u8; 64 * 1024];
    loop {
        let read = file.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        hasher.update(&chunk[..read]);
    }
    let bytes = hasher.finalize();
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Returns the checksum a `sha256sum` list gives for the file `name`, in lowercase. A binary
/// mark (`*`) before the name is accepted.
pub fn listed(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, file) = line.trim().split_once(char::is_whitespace)?;
        let file = file.trim_start().trim_start_matches('*');
        let hex = hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit());
        (file == name && hex).then(|| hash.to_ascii_lowercase())
    })
}

/// Checks the file at `path` against the checksum `sums` lists for `name`.
pub fn verify(path: &Path, sums: &str, name: &str) -> Result<(), Failure> {
    let wanted = listed(sums, name).ok_or(Failure::Checksum)?;
    match digest(path)? == wanted {
        true => Ok(()),
        false => Err(Failure::Checksum),
    }
}
