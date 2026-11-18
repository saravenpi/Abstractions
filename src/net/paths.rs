//! Where the network keeps its files, next to the settings file: the servers the player trusts,
//! the player's identity key and the certificate of a world opened to LAN.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bevy::prelude::*;

/// Length of the player's identity key in bytes.
const KEY_BYTES: usize = 32;

/// The folder of the network files, the settings folder unless a test gives another.
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct NetDir(pub PathBuf);

impl Default for NetDir {
    /// Returns the settings folder.
    fn default() -> Self {
        Self(crate::settings::config_dir())
    }
}

impl NetDir {
    /// Returns the file of the servers the player trusts.
    pub fn known(&self) -> PathBuf {
        self.0.join("known_servers.toml")
    }

    /// Returns the folder of the certificate a world opened to LAN shows.
    pub fn host(&self) -> PathBuf {
        self.0.join("host")
    }

    /// Returns the player's identity key, made at random and stored the first time. The
    /// server finds the player's record by it.
    pub fn identity(&self) -> io::Result<[u8; KEY_BYTES]> {
        let path = self.0.join("identity.key");
        if let Ok(bytes) = fs::read(&path)
            && let Ok(key) = <[u8; KEY_BYTES]>::try_from(bytes.as_slice())
        {
            return Ok(key);
        }
        let key: [u8; KEY_BYTES] = rand::random();
        store(&path, &key)?;
        Ok(key)
    }
}

/// Writes `key` to `path`, readable by the owner only on unix.
fn store(path: &Path, key: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)?.write_all(key)
}
