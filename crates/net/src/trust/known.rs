//! The servers a player already trusts: a small TOML file of addresses and fingerprints.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

/// The fingerprint each known address showed the first time, kept in a TOML file such as
/// `known_servers.toml`, one `"address" = "hex fingerprint"` line per server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownServers {
    /// File the list is read from and saved to.
    path: PathBuf,
    /// Fingerprint in lowercase hex, by address.
    servers: BTreeMap<String, String>,
}

impl KnownServers {
    /// Reads the list at `path`. A missing file is an empty list; a damaged one is an error, so
    /// a player never silently forgets the keys it trusts.
    pub fn load(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let servers = match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error),
        };
        Ok(Self { path, servers })
    }

    /// Returns the fingerprint remembered for `address`, if any.
    pub fn get(&self, address: &str) -> Option<[u8; 32]> {
        self.servers.get(address).and_then(|text| unhex(text))
    }

    /// Remembers `fingerprint` for `address`, replacing what was there. Call [`Self::save`]
    /// to keep it.
    pub fn insert(&mut self, address: &str, fingerprint: [u8; 32]) {
        self.servers.insert(address.to_string(), hex(&fingerprint));
    }

    /// Writes the list through a temporary file, so a crash never leaves half a file.
    pub fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let text = toml::to_string(&self.servers).map_err(io::Error::other)?;
        let temporary = self.path.with_extension("tmp");
        fs::write(&temporary, text)?;
        fs::rename(&temporary, &self.path)
    }
}

/// Returns `bytes` as lowercase hex.
fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Reads 64 hex digits back into bytes, or nothing when the text is not that.
fn unhex(text: &str) -> Option<[u8; 32]> {
    let text = text.as_bytes();
    if text.len() != 64 {
        return None;
    }
    let mut out = [0; 32];
    for (byte, pair) in out.iter_mut().zip(text.chunks(2)) {
        *byte = u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?;
    }
    Some(out)
}
