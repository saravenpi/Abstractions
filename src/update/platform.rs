//! The builds a release ships and the file of each that the game installs by itself.

use super::release::{Asset, Release};

/// A platform the releases are built for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Linux on x86_64: a plain binary.
    Linux,
    /// Windows on x86_64: a plain `.exe`.
    Windows,
    /// macOS, Apple Silicon and Intel: a disk image holding the application.
    Mac,
}

impl Platform {
    /// Returns the platform of a system and processor as Rust names them, when releases have a
    /// build for it.
    pub fn of(os: &str, arch: &str) -> Option<Self> {
        match (os, arch) {
            ("linux", "x86_64") => Some(Platform::Linux),
            ("windows", "x86_64") => Some(Platform::Windows),
            ("macos", _) => Some(Platform::Mac),
            _ => None,
        }
    }

    /// Returns the platform this game was built for, when releases have a build for it.
    pub fn current() -> Option<Self> {
        Self::of(std::env::consts::OS, std::env::consts::ARCH)
    }

    /// Returns the platform part of the release file names.
    pub fn slug(self) -> &'static str {
        match self {
            Platform::Linux => "linux-x86_64",
            Platform::Windows => "windows-x86_64",
            Platform::Mac => "macos-universal",
        }
    }

    /// Returns the name of the file the game installs from the release tagged `tag`.
    pub fn payload(self, tag: &str) -> String {
        match self {
            Platform::Linux => format!("abstractions-{tag}-linux-x86_64"),
            Platform::Windows => format!("abstractions-{tag}-windows-x86_64.exe"),
            Platform::Mac => format!("Abstractions-{tag}-macos-universal.dmg"),
        }
    }

    /// Returns the name of the checksum list of this platform in the release tagged `tag`.
    pub fn sums(self, tag: &str) -> String {
        format!("SHA256SUMS-{}-{tag}.txt", self.slug())
    }
}

/// Returns the file to install and the checksum list that covers it, when the release has both
/// for `platform`.
pub fn pick(release: &Release, platform: Platform) -> Option<(Asset, Asset)> {
    let named = |name: String| release.assets.iter().find(|asset| asset.name == name);
    let payload = named(platform.payload(&release.tag))?;
    let sums = named(platform.sums(&release.tag))?;
    Some((payload.clone(), sums.clone()))
}

/// Returns the program, and its first arguments, that opens a file or an address with the
/// default application of the system named `os`.
pub fn opener(os: &str) -> (&'static str, &'static [&'static str]) {
    match os {
        "windows" => ("cmd", &["/C", "start", ""]),
        "macos" => ("open", &[]),
        _ => ("xdg-open", &[]),
    }
}
