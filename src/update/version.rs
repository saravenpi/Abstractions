//! Version numbers: reading `major.minor.patch` and telling which of two is newer.

use std::fmt;

/// A released version. Versions order by major, then minor, then patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    /// Major number.
    pub major: u64,
    /// Minor number.
    pub minor: u64,
    /// Patch number.
    pub patch: u64,
}

impl fmt::Display for Version {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Reads one part of a version: digits only, so that a sign or a space is refused.
fn number(part: &str) -> Option<u64> {
    let digits = !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    digits.then(|| part.parse().ok()).flatten()
}

/// Reads a version such as `0.9.1` or `v0.9.1`. A pre-release or build suffix (`-rc.1`, `+7`) is
/// dropped. Anything that is not three whole numbers is no version.
pub fn parse(text: &str) -> Option<Version> {
    let bare = text.trim();
    let bare = bare.strip_prefix(['v', 'V']).unwrap_or(bare);
    let core = bare.split(['-', '+']).next()?;
    let mut numbers = core.split('.').map(number);
    let (major, minor, patch) = (numbers.next()??, numbers.next()??, numbers.next()??);
    let version = Version {
        major,
        minor,
        patch,
    };
    numbers.next().is_none().then_some(version)
}

/// Returns whether `candidate` is a newer version than `current`. Text that is not a version is
/// never newer, and nothing is newer than it.
pub fn newer(current: &str, candidate: &str) -> bool {
    match (parse(current), parse(candidate)) {
        (Some(current), Some(candidate)) => candidate > current,
        _ => false,
    }
}
